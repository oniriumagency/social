//! Corpus de identificadores (`spec/vectors/ids.json`): eventos generados con semilla fija
//! y su `id` calculado en nativo. Los ejecutores en WASM (Node y navegador) comprueban que
//! calculan el mismo `id` para cada uno (`docs/14` §4: nativo y WASM coinciden en toda
//! entrada generada).
//!
//! El generador pseudoaleatorio es SplitMix64: solo produce datos de prueba, nunca
//! material criptográfico. Las llaves son las públicas de prueba de los vectores.

use std::collections::BTreeMap;

use social_crypto::{SecretKey32, SigningKey};

use crate::json::MAX_SAFE_INT;
use crate::{CoreError, Json, Tag, UnsignedEvent};

/// Número de eventos del corpus.
pub const CORPUS_SIZE: usize = 128;
const SEED: u64 = 0x5eed_0f1d_5c0d_e000;

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Entero en `0..n` (`n > 0`). El sesgo es irrelevante para datos de prueba.
    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % (n as u64)).unwrap_or(0)
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

/// Caracteres que ejercitan el escape de JCS, el orden UTF-16 y el UTF-8 de varios bytes.
const CHARS: &[char] = &[
    'a',
    'z',
    'A',
    '0',
    ' ',
    '"',
    '\\',
    '/',
    '\n',
    '\r',
    '\t',
    '\u{0}',
    '\u{1}',
    '\u{8}',
    '\u{c}',
    '\u{1f}',
    '\u{7f}',
    '\u{80}',
    'ñ',
    'é',
    'ö',
    '€',
    '\u{2028}',
    '\u{fb33}',
    '\u{fffd}',
    '\u{ffff}',
    '日',
    '😀',
    '\u{10ffff}',
    '\u{1d11e}',
];
const TOPIC_CHARS: &[char] = &['a', 'm', 'z', '0', '9', '_', 'ñ', 'é', 'ü', '日'];

fn string(rng: &mut SplitMix64, max: usize) -> String {
    let len = rng.below(max + 1);
    (0..len).map(|_| *rng.pick(CHARS)).collect()
}

fn int(rng: &mut SplitMix64) -> i64 {
    let span = u64::try_from(MAX_SAFE_INT).unwrap_or(0) * 2 + 1;
    let n = i64::try_from(rng.next() % span).unwrap_or(0) - MAX_SAFE_INT;
    match rng.below(4) {
        0 => n,
        1 => n % 1000,
        2 => *rng.pick(&[0, -1, 1, MAX_SAFE_INT, -MAX_SAFE_INT]),
        _ => n >> 20,
    }
}

fn value(rng: &mut SplitMix64, depth: usize) -> Json {
    let leaf = depth == 0 || rng.below(3) == 0;
    if leaf {
        return match rng.below(3) {
            0 => Json::Bool(rng.below(2) == 0),
            1 => Json::Int(int(rng)),
            _ => Json::String(string(rng, 12)),
        };
    }
    if rng.below(2) == 0 {
        let n = rng.below(4);
        Json::Array((0..n).map(|_| value(rng, depth - 1)).collect())
    } else {
        Json::Object(object(rng, depth - 1))
    }
}

fn object(rng: &mut SplitMix64, depth: usize) -> BTreeMap<String, Json> {
    let n = rng.below(5);
    (0..n)
        .map(|_| (string(rng, 6), value(rng, depth)))
        .collect()
}

fn test_key(n: u8) -> SigningKey {
    let mut seed = [0u8; 32];
    seed[0] = n;
    seed[31] = 0x1d;
    SigningKey::from_seed(&SecretKey32::from_bytes(seed))
}

/// Genera `spec/vectors/ids.json`.
///
/// # Errors
/// Solo si el generador produjera un evento que el núcleo rechaza.
pub fn generate_ids_suite() -> Result<String, CoreError> {
    let mut rng = SplitMix64(SEED);
    let kinds = ["test.ids", "post.create_x", "a.b.c", "z9.y_8"];
    let mut accept = Vec::with_capacity(CORPUS_SIZE);
    for i in 0..CORPUS_SIZE {
        let key = test_key(1 + u8::try_from(i % 4).unwrap_or(0));
        let account = if rng.below(2) == 0 {
            key.verifying_key().to_bytes()
        } else {
            test_key(9).verifying_key().to_bytes()
        };
        let tags = (0..rng.below(4))
            .map(|_| {
                if rng.below(2) == 0 {
                    let len = 1 + rng.below(8);
                    Tag::Topic((0..len).map(|_| *rng.pick(TOPIC_CHARS)).collect())
                } else {
                    Tag::Mention(
                        test_key(10 + u8::try_from(rng.below(3)).unwrap_or(0))
                            .verifying_key()
                            .to_bytes(),
                    )
                }
            })
            .collect();
        let event = UnsignedEvent {
            account,
            signer: key.verifying_key().to_bytes(),
            created_at: int(&mut rng).abs(),
            kind: (*rng.pick(&kinds)).to_owned(),
            tags,
            content: object(&mut rng, 3),
        }
        .sign(&key)?;
        let mut entry = BTreeMap::new();
        entry.insert("name".to_owned(), Json::String(format!("generado {i}")));
        entry.insert(
            "event".to_owned(),
            Json::String(event.to_json().to_canonical_string()),
        );
        let mut expect = BTreeMap::new();
        expect.insert("valid".to_owned(), Json::Bool(true));
        expect.insert("id".to_owned(), Json::String(hex::encode(event.id())));
        entry.insert("expect".to_owned(), Json::Object(expect));
        accept.push(Json::Object(entry));
    }
    let mut suite = BTreeMap::new();
    suite.insert("suite".to_owned(), Json::from("ids"));
    suite.insert("protocol".to_owned(), Json::Int(crate::PROTOCOL_VERSION));
    suite.insert(
        "note".to_owned(),
        Json::from("Generado por `pnpm vectors:gen` con semilla fija. El id lo calcula el núcleo nativo; WASM debe coincidir (docs/14 §4)."),
    );
    suite.insert("accept".to_owned(), Json::Array(accept));
    let mut text = Json::Object(suite).to_canonical_string();
    text.push('\n');
    Ok(text)
}
