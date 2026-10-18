//! Cœur de wikik : transforme un projet (code + zone Projet) en index requêtable.
//!
//! Aucune entrée/sortie réseau ici : la CLI, le serveur et un futur build WASM
//! s'appuient tous sur cette bibliothèque (ADR-0005).
//!
//! @adr 2, 5

pub mod atelier;
pub mod check;
pub mod config;
pub mod git;
pub mod hash;
pub mod index;
pub mod init;
pub mod lang;
pub mod modele;
pub mod projet;
pub mod query;
pub mod relu;
pub mod reseau;
pub mod resolution;
pub mod stats;
pub mod wiki;

pub use index::{Index, Rapport};

/// Version du schéma de l'index ; un index d'une autre version est reconstruit (ADR-0003).
/// @relu 2026-10-17 719e028c
pub const SCHEMA: u32 = 2;

/// Version des règles d'extraction (hash, identifiants, résolution). À incrémenter à chaque
/// changement qui modifie le contenu de l'index sans changer les fichiers : sinon l'incrémental
/// garderait des valeurs calculées par l'ancien algorithme et `check` comparerait des hash incompatibles.
/// @relu 2026-10-17 d9a2eed6
pub const VERSION_EXTRACTION: u32 = 7;
