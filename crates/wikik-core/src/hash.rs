//! Empreintes SHA-256 tronquées à 16 caractères hexadécimaux (SPEC-02).

use sha2::{Digest, Sha256};

/// Empreinte de `donnees` sur 64 bits : assez pour distinguer les versions d'un symbole,
/// assez court pour rester lisible dans un dump ou un diagnostic.
pub fn empreinte(donnees: &[u8]) -> String {
    let h = Sha256::digest(donnees);
    h.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// Empreinte d'une suite de morceaux séparés par `\0`, pour que `["ab", "c"]` et `["a", "bc"]` diffèrent.
pub fn empreinte_morceaux<'a>(morceaux: impl IntoIterator<Item = &'a str>) -> String {
    let mut h = Sha256::new();
    for m in morceaux {
        h.update(m.as_bytes());
        h.update([0u8]);
    }
    h.finalize()
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}
