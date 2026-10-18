//! Cahier de réception (SPEC-22, SPEC-23) : carte des pages et remarques de l'utilisateur.

pub mod carte;
pub mod remarques;

use crate::index::Index;
use anyhow::Result;
use serde::Serialize;
use ts_rs::TS;

/// État servi à l'écran : la carte et les remarques.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct EtatReception {
    pub pages: Vec<carte::Page>,
    pub remarques: Vec<remarques::Remarque>,
    pub fournisseur: String,
}

/// État du cahier de réception pour l'écran.
pub fn etat(index: &Index, fournisseur: &str) -> Result<EtatReception> {
    Ok(EtatReception {
        pages: carte::lire(index),
        remarques: remarques::lister(index)?,
        fournisseur: fournisseur.into(),
    })
}
