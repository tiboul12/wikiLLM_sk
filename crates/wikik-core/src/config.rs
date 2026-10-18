//! Lecture de `wikik.toml`, entièrement optionnel : sans fichier, les valeurs par défaut s'appliquent (EF-004).
//!
//! @req EF-004

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Configuration d'un projet.
/// @relu 2026-10-18 341b2356
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub index: ConfigIndex,
    pub projet: ConfigProjet,
    pub check: ConfigCheck,
    pub equipe: ConfigEquipe,
    pub atelier: ConfigAtelier,
    pub serveur: ConfigServeur,
    pub reception: ConfigReception,
}

/// Section `[reception]` : dossier du cahier de réception (carte des pages, remarques, images ; SPEC-23).
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigReception {
    pub dossier: String,
}

impl Default for ConfigReception {
    fn default() -> Self {
        Self {
            dossier: "docs/reception".into(),
        }
    }
}

impl Config {
    /// Dossiers dont les `.md` sont des documents de l'index : zone Projet, atelier, réception.
    pub fn zones_documents(&self) -> [&str; 3] {
        [
            self.projet.dossier.trim_end_matches('/'),
            self.atelier.dossier.trim_end_matches('/'),
            self.reception.dossier.trim_end_matches('/'),
        ]
    }
}

/// Section `[serveur]` : `reseau = true` fait écouter `wikik serve` et `wikik atelier` sur le réseau
/// local sans option à retaper (équivaut à `--reseau`). Choix explicite : l'atelier écrit dans le projet.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigServeur {
    pub reseau: bool,
}

/// Section `[atelier]` : dossier des artefacts, adresses des deux serveurs, version majeure documentée,
/// LLM (SPEC-17 à 21).
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigAtelier {
    /// Dossier des artefacts de l'atelier, relatif à la racine.
    pub dossier: String,
    /// Adresse de l'atelier, pour les liens « ouvrir dans l'atelier » du wiki.
    pub url: String,
    /// Adresse du wiki, pour les liens « voir dans le wiki » de l'atelier.
    pub url_wiki: String,
    /// Version majeure que l'atelier documente (dossier `docs/projet/majeures/<version>/`).
    pub version: String,
    pub llm: ConfigLlm,
}

impl Default for ConfigAtelier {
    fn default() -> Self {
        Self {
            dossier: "docs/atelier".into(),
            url: "http://127.0.0.1:8485".into(),
            url_wiki: "http://127.0.0.1:8484".into(),
            version: "v1".into(),
            llm: ConfigLlm::default(),
        }
    }
}

/// Section `[atelier.llm]` : commande du fournisseur (SPEC-19). Les jetons `{consigne}`, `{schema}`
/// et `{mcp}` des arguments sont remplacés à chaque appel.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigLlm {
    /// `cli` (défaut) ou `factice` (tests, démonstration sans LLM).
    pub fournisseur: String,
    pub commande: String,
    pub arguments: Vec<String>,
    /// Délai maximum d'un appel, en secondes.
    pub delai: u64,
}

impl Default for ConfigLlm {
    fn default() -> Self {
        let a = |s: &str| s.to_string();
        Self {
            fournisseur: a("cli"),
            commande: a("claude"),
            arguments: [
                "-p",
                "{consigne}",
                "--mcp-config",
                "{mcp}",
                "--strict-mcp-config",
                "--restricted",
                "--tools",
                "",
                "--allowedTools",
                "mcp__wikik",
                "--output-format",
                "stream-json",
                "--verbose",
                "--json-schema",
                "{schema}",
                "--no-session-persistence",
            ]
            .iter()
            .map(|s| a(s))
            .collect(),
            delai: 180,
        }
    }
}

/// Section `[equipe]` : qui valide les documents produit (SPEC-16 R4).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigEquipe {
    /// Comptes ou équipes GitHub (`@tibo`, `@org/produit`) dont l'approbation est exigée sur les
    /// BDR, cahiers des charges et exigences (fichier `CODEOWNERS`).
    pub validateurs: Vec<String>,
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
            langages: vec![
                "rust".into(),
                "typescript".into(),
                "javascript".into(),
                "svelte".into(),
            ],
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
