//! Langages supportés : un module par langage derrière le trait [`Langage`] (SPEC-03 R1).

pub mod rust;
pub mod svelte;
pub mod typescript;

use crate::modele::ExtraitFichier;

/// Extraction d'un fichier source vers le modèle commun de l'index.
pub trait Langage: Send + Sync {
    /// Nom court, tel qu'il apparaît dans `wikik.toml` et dans l'index (`rust`).
    fn nom(&self) -> &'static str;
    /// Extensions sans point (`rs`).
    fn extensions(&self) -> &'static [&'static str];
    /// Ne doit jamais échouer : un fichier mal formé donne un extrait partiel (ENF-006).
    fn extraire(&self, chemin: &str, source: &[u8]) -> ExtraitFichier;
}

/// Langages activés par la configuration ; un nom inconnu est ignoré.
/// @relu 2026-10-17 38cd801b
pub fn actives(noms: &[String]) -> Vec<Box<dyn Langage>> {
    let mut v: Vec<Box<dyn Langage>> = vec![];
    for n in noms {
        match n.as_str() {
            "rust" => v.push(Box::new(rust::Rust)),
            "typescript" => v.push(Box::new(typescript::TypeScript)),
            "javascript" => v.push(Box::new(typescript::JavaScript)),
            "svelte" => v.push(Box::new(svelte::Svelte)),
            _ => {}
        }
    }
    v
}
