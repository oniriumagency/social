//! Validación universal de eventos (`docs/06` §3, pasos 1 a 6, más los límites del tipo).
//!
//! La aplican por igual cliente, pasarela, indexador y nodo (I-9). El estado que necesita
//! (llaves del registro y de operadores, delegaciones y atestaciones) se consulta a través
//! de [`ValidationContext`], que cada participante implementa con su propio almacén: en
//! memoria en el cliente y en pruebas, PostgreSQL en la pasarela (F1-T03).
//!
//! Los pasos 7 (reloj de la ingesta) y los límites de frecuencia no son universales: los
//! aplica solo quien ingesta.

use crate::kinds::{self, SignerRole};
use crate::{CoreError, ErrorCode, Event};

/// Delegación de un dispositivo (`device.grant`, `docs/06` §4), tal como la proyecta el almacén.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceGrant {
    /// Tipos que el dispositivo puede firmar.
    pub caps: Vec<String>,
    /// `created_at` del `device.grant`: la delegación vale desde ese instante.
    pub granted_at: i64,
    /// Caduca en este instante (excluido).
    pub expires_at: i64,
    /// `created_at` del `device.revoke`, si lo hubo. Efecto inmediato desde ese instante.
    pub revoked_at: Option<i64>,
}

/// Atestación vigente de una cuenta (`account.attest`). Solo el nivel, nunca los métodos (I-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attestation {
    /// Nivel de confianza: 1, 2 o 3.
    pub level: u8,
    /// Vale desde este instante (incluido).
    pub valid_from: i64,
    /// Caduca en este instante (excluido).
    pub expires_at: i64,
}

/// Estado que la validación universal necesita consultar.
pub trait ValidationContext {
    /// La llave pertenece al registro de humanidad (publicada en `/v1/info`).
    fn is_registry_key(&self, key: &[u8; 32]) -> bool;
    /// La llave pertenece a un operador de moderación.
    fn is_operator_key(&self, key: &[u8; 32]) -> bool;
    /// Delegación de `device` por `account`, si existe.
    fn device_grant(&self, account: &[u8; 32], device: &[u8; 32]) -> Option<DeviceGrant>;
    /// Atestación de `account` vigente en el instante `at`, si existe.
    fn attestation(&self, account: &[u8; 32], at: i64) -> Option<Attestation>;
}

/// Nivel mínimo para publicar contenido con imágenes (`docs/05` §4).
pub const MIN_LEVEL_MEDIA: u8 = 2;

/// Analiza y valida un evento recibido: pasos 1 a 6 de `docs/06` §3 y límites del tipo.
///
/// # Errors
/// El [`CoreError`] del primer paso que falle, con su código estable.
pub fn validate_event(bytes: &[u8], ctx: &dyn ValidationContext) -> Result<Event, CoreError> {
    // Pasos 1 a 4: tamaño, esquema de la envoltura, id y firma.
    let event = Event::parse_and_verify(bytes)?;
    validate_parsed(&event, ctx)?;
    Ok(event)
}

/// Pasos 2 (esquema del tipo), 5 y 6, y límites del tipo, sobre un evento ya verificado.
///
/// # Errors
/// El [`CoreError`] del primer paso que falle.
pub fn validate_parsed(event: &Event, ctx: &dyn ValidationContext) -> Result<(), CoreError> {
    let account = event.account();
    let signer = event.signer();
    let created_at = event.created_at();
    let spec = kinds::spec(event.kind());

    // Paso 2 (tipo): esquema del contenido y límites del tipo (`docs/06` §4).
    let facts = kinds::check_content(
        event.kind(),
        created_at,
        &hex::encode(signer),
        event.content(),
    )?;

    // Paso 5: quién firma.
    let role = spec.map_or(SignerRole::RootOrDevice, |s| s.signer);
    let is_root = signer == account;
    let role_error = |msg: &str| CoreError::new(ErrorCode::SignerRole, msg.to_owned());
    match role {
        SignerRole::Registry => {
            if !is_root || !ctx.is_registry_key(signer) {
                return Err(role_error("este tipo solo lo firma el registro"));
            }
            return Ok(());
        }
        SignerRole::Operator => {
            if !is_root || !ctx.is_operator_key(signer) {
                return Err(role_error("este tipo solo lo firma un operador"));
            }
            return Ok(());
        }
        SignerRole::Root if !is_root => {
            return Err(role_error("este tipo solo lo firma la llave raíz"));
        }
        SignerRole::Device if is_root => {
            return Err(role_error(
                "este tipo lo firma un dispositivo delegado, no la raíz",
            ));
        }
        SignerRole::RootOrRegistry if !is_root && ctx.is_registry_key(signer) => {
            // Rotación de recuperación firmada por el registro (`docs/05`): sin atestación.
            return Ok(());
        }
        _ => {}
    }
    if !is_root {
        check_delegation(ctx, account, signer, event.kind(), created_at)?;
    }

    // Paso 6: atestación vigente y nivel suficiente.
    let needs_attestation = spec.is_none_or(kinds::KindSpec::needs_attestation);
    if needs_attestation {
        let att = ctx.attestation(account, created_at).filter(|a| {
            a.valid_from <= created_at && created_at < a.expires_at && (1..=3).contains(&a.level)
        });
        let Some(att) = att else {
            return Err(CoreError::new(
                ErrorCode::AttestationMissing,
                "la cuenta no tiene atestación vigente en created_at",
            ));
        };
        if facts.has_media && att.level < MIN_LEVEL_MEDIA {
            return Err(CoreError::new(
                ErrorCode::LevelInsufficient,
                "adjuntar imágenes exige nivel 2 o superior",
            ));
        }
    }
    Ok(())
}

fn check_delegation(
    ctx: &dyn ValidationContext,
    account: &[u8; 32],
    signer: &[u8; 32],
    kind: &str,
    created_at: i64,
) -> Result<(), CoreError> {
    let Some(grant) = ctx.device_grant(account, signer) else {
        return Err(CoreError::new(
            ErrorCode::DelegationMissing,
            "signer no tiene device.grant de account",
        ));
    };
    if grant.revoked_at.is_some_and(|r| created_at >= r) {
        return Err(CoreError::new(
            ErrorCode::DelegationRevoked,
            "delegación revocada",
        ));
    }
    if created_at < grant.granted_at || created_at >= grant.expires_at {
        return Err(CoreError::new(
            ErrorCode::DelegationExpired,
            "delegación fuera de vigencia en created_at",
        ));
    }
    if !grant.caps.iter().any(|c| c == kind) {
        return Err(CoreError::new(
            ErrorCode::DelegationScope,
            "la delegación no cubre este tipo",
        ));
    }
    Ok(())
}
