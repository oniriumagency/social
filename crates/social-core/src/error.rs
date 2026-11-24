use core::fmt;

/// Códigos de error estables del protocolo (`docs/06` §6). Forman parte de la API pública
/// y de los vectores dorados: **no se renombran ni se reutilizan**. Añadir uno exige
/// actualizar `docs/06` y los vectores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorCode {
    /// El evento serializado supera `MAX_EVENT_BYTES` (paso 1).
    EventTooLarge,
    /// No es JSON válido en UTF-8 (paso 2).
    JsonInvalid,
    /// Un objeto JSON repite una clave (paso 2).
    JsonDuplicateKey,
    /// Aparece un número decimal o en notación exponencial (paso 2).
    JsonFloat,
    /// Aparece `null` (paso 2).
    JsonNull,
    /// Un entero fuera de ±(2⁵³ − 1) (paso 2).
    JsonIntRange,
    /// La envoltura o el contenido no cumplen el esquema (paso 2).
    Schema,
    /// La envoltura tiene un campo desconocido (paso 2).
    UnknownField,
    /// Versión de envoltura no admitida (paso 2).
    VersionUnsupported,
    /// `account` o `signer` no es una llave pública Ed25519 válida (paso 2).
    PublicKeyInvalid,
    /// `id` distinto del calculado (paso 3).
    IdMismatch,
    /// Firma inválida o no canónica para `signer` (paso 4).
    SigInvalid,
}

impl ErrorCode {
    /// Todos los códigos, para documentación y pruebas.
    pub const ALL: &'static [Self] = &[
        Self::EventTooLarge,
        Self::JsonInvalid,
        Self::JsonDuplicateKey,
        Self::JsonFloat,
        Self::JsonNull,
        Self::JsonIntRange,
        Self::Schema,
        Self::UnknownField,
        Self::VersionUnsupported,
        Self::PublicKeyInvalid,
        Self::IdMismatch,
        Self::SigInvalid,
    ];

    /// Código textual estable, el que viaja en la API: `{"error":{"code":"E_..."}}`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EventTooLarge => "E_EVENT_TOO_LARGE",
            Self::JsonInvalid => "E_JSON_INVALID",
            Self::JsonDuplicateKey => "E_JSON_DUPLICATE_KEY",
            Self::JsonFloat => "E_JSON_FLOAT",
            Self::JsonNull => "E_JSON_NULL",
            Self::JsonIntRange => "E_JSON_INT_RANGE",
            Self::Schema => "E_SCHEMA",
            Self::UnknownField => "E_UNKNOWN_FIELD",
            Self::VersionUnsupported => "E_VERSION_UNSUPPORTED",
            Self::PublicKeyInvalid => "E_PUBKEY_INVALID",
            Self::IdMismatch => "E_ID_MISMATCH",
            Self::SigInvalid => "E_SIG_INVALID",
        }
    }

    /// Interpreta un código textual.
    #[must_use]
    pub fn parse(code: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.as_str() == code)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error del núcleo: un código estable y un detalle legible para desarrolladores.
/// El detalle nunca contiene material secreto ni datos personales; solo nombres de campo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreError {
    /// Código estable.
    pub code: ErrorCode,
    /// Detalle técnico (por ejemplo, la ruta del campo que falla).
    pub detail: String,
}

impl CoreError {
    pub(crate) fn new(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for CoreError {}
