//! Cœur de wikik : transforme un projet (code + zone Projet) en index requêtable.
//!
//! Aucune entrée/sortie réseau ici : la CLI, le serveur et un futur build WASM
//! s'appuient tous sur cette bibliothèque (ADR-0005).
//!
//! @adr 2, 5

/// Version du schéma de l'index ; un index d'une autre version est reconstruit (ADR-0003).
pub const SCHEMA: u32 = 1;
