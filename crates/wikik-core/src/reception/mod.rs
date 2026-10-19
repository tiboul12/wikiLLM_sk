//! Cahier de réception (SPEC-22, SPEC-23) : carte des pages et remarques de l'utilisateur.

pub mod carte;
pub mod remarques;

use crate::index::Index;
use anyhow::Result;
use serde::Serialize;
use ts_rs::TS;

/// État servi à l'écran : la carte, les remarques (versions remplacées comprises, pour l'historique) et
/// l'adresse de l'application reçue (saisie, ou déduite du projet).
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct EtatReception {
    pub pages: Vec<carte::Page>,
    pub remarques: Vec<remarques::Remarque>,
    pub fournisseur: String,
    /// Adresse de l'application reçue, saisie par Tibo, pour les liens « ouvrir dans l'application ».
    pub url_application: Option<String>,
    /// Adresse déduite du projet quand aucune n'est saisie (`/` : le serveur lui-même).
    pub url_detectee: Option<String>,
}

/// État du cahier de réception pour l'écran : la carte enregistrée fusionnée avec le scan du moment (rien
/// n'est écrit).
pub fn etat(index: &Index, fournisseur: &str) -> Result<EtatReception> {
    Ok(EtatReception {
        // La carte à jour : l'enregistrée (corrections comprises) fusionnée avec ce que le scan trouve
        // maintenant, sans rien écrire ; elle est réenregistrée au prochain geste.
        pages: carte::fusionner(&carte::lire(index), carte::detecter(index)?),
        remarques: remarques::lister(index)?,
        fournisseur: fournisseur.into(),
        url_application: carte::url_application(index),
        url_detectee: carte::url_detectee(index),
    })
}
