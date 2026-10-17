//! Lecture de `wikik.toml`, entièrement optionnel : sans fichier, les valeurs par défaut s'appliquent (EF-004).
//!
//! @req EF-004

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Configuration d'un projet.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub index: ConfigIndex,
    pub projet: ConfigProjet,
    pub check: ConfigCheck,
}

/// Section `[index]` : ce qui est indexé.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigIndex {
    /// Motifs glob (relatifs à la racine) exclus de l'index, en plus du `.gitignore`.
    pub exclure: Vec<String>,
    pub langages: Vec<String>,
}

impl Default for ConfigIndex {
    fn default() -> Self {
        Self {
            exclure: vec![],
            langages: vec!["rust".into()],
        }
    }
}

/// Section `[projet]` : où vit la zone Projet.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigProjet {
    /// Dossier de la zone Projet, relatif à la racine.
    pub dossier: String,
}

impl Default for ConfigProjet {
    fn default() -> Self {
        Self {
            dossier: "docs/projet".into(),
        }
    }
}

/// Section `[check]` : exigences de doc et sévérités (SPEC-04).
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigCheck {
    /// `pub`, `pub-crate` ou `tout` : à partir de quelle visibilité un bloc est exigé (SPEC-01 R5).
    pub seuil_visibilite: String,
    /// Sévérité par code de diagnostic : `erreur`, `avertissement`, `info` ou `ignore`.
    pub severites: BTreeMap<String, String>,
}

impl Default for ConfigCheck {
    fn default() -> Self {
        Self {
            seuil_visibilite: "pub".into(),
            severites: BTreeMap::new(),
        }
    }
}

impl Config {
    /// Charge `<racine>/wikik.toml` s'il existe ; une clé inconnue est une erreur, pour ne pas ignorer une faute de frappe.
    pub fn charger(racine: &Path) -> Result<Config> {
        let chemin = racine.join("wikik.toml");
        if !chemin.exists() {
            return Ok(Config::default());
        }
        let texte = std::fs::read_to_string(&chemin)
            .with_context(|| format!("lecture de {}", chemin.display()))?;
        toml::from_str(&texte)
            .with_context(|| format!("wikik.toml invalide ({})", chemin.display()))
    }
}
