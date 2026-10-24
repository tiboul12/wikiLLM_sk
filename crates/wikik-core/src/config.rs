//! Lecture de `wikik.toml`, entièrement optionnel : sans fichier, les valeurs par défaut s'appliquent (EF-004).
//!
//! @req EF-004

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Configuration d'un projet.
/// @relu 2026-10-24 875fe1a8
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
    pub ia: ConfigIa,
    pub semantique: ConfigSemantique,
}

/// Section `[semantique]` : recherche par le sens (EF-173, ADR-0020), par les embeddings d'Ollama.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigSemantique {
    /// Modèle d'embeddings d'Ollama.
    pub modele: String,
    /// Adresse d'Ollama ; vide : celle du connecteur Ollama, sinon celle de Shelley ou `localhost:11434`.
    pub url: String,
}

impl Default for ConfigSemantique {
    /// `nomic-embed-text`, à l'adresse d'Ollama déduite.
    fn default() -> Self {
        Self {
            modele: "nomic-embed-text".into(),
            url: String::new(),
        }
    }
}

/// Section `[ia]` : agent de développement (EF-147). Les consignes d'avant la v8 y restent lisibles, mais
/// les consignes des LLM sont désormais des skills (ADR-0017) : une valeur trouvée ici passe dans son
/// skill à la première écriture des Paramètres.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigIa {
    /// Méthode de lecture du wiki : instructions du serveur MCP (tout client, Claude Code compris) et
    /// préambule des demandes de l'atelier et de la réception.
    pub consigne_lecture: String,
    /// Consigne tapée dans le terminal par « Traiter les remarques » (EF-148).
    pub consigne_traitement: String,
    /// Commande qui lance l'agent de développement dans le terminal intégré.
    pub agent: String,
}

impl Default for ConfigIa {
    /// Consignes par défaut : interroger wikik avant de lire le code, traiter la réception par un plan validé puis lot par lot ; agent `claude`.
    fn default() -> Self {
        Self {
            consigne_lecture: "Interroger wikik avant d'ouvrir des fichiers source : carte en début de session, symbole puis impact avant de modifier du code ; document pour lire un besoin, une exigence ou une décision par son identifiant.".into(),
            consigne_traitement: "Traite les remarques ouvertes du cahier de réception de ce projet (outil MCP reception de wikik). Commence par un plan : regroupe les remarques par lot, donne l'ordre et ce que tu vas changer pour chacune, et attends ma validation avant de coder. Ensuite, lot par lot : corrige, vérifie, commite, puis enregistre chaque remarque avec reception_traiter (statut, explication en une phrase, commit).".into(),
            agent: "claude".into(),
        }
    }
}

/// Section `[reception]` : dossier du cahier de réception (carte des pages, remarques, images ; SPEC-23).
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigReception {
    pub dossier: String,
}

impl Default for ConfigReception {
    /// Cahier de réception rangé dans `docs/reception`.
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

/// Section `[serveur]` : `reseau = true` fait écouter le serveur (`wikik`) sur le réseau local sans
/// option à retaper (équivaut à `--reseau`). Choix explicite : le serveur écrit dans le projet.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigServeur {
    pub reseau: bool,
}

/// Section `[atelier]` : dossier des artefacts, version majeure documentée, LLM (SPEC-17 à 21).
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigAtelier {
    /// Dossier des artefacts de l'atelier, relatif à la racine.
    pub dossier: String,
    /// Version majeure que l'atelier documente (dossier `docs/projet/majeures/<version>/`).
    pub version: String,
    pub llm: ConfigLlm,
}

impl Default for ConfigAtelier {
    /// Atelier rangé dans `docs/atelier`, version `v1`, LLM par défaut.
    fn default() -> Self {
        Self {
            dossier: "docs/atelier".into(),
            version: "v1".into(),
            llm: ConfigLlm::default(),
        }
    }
}

/// Section `[atelier.llm]` : le LLM de l'atelier, de la réception et de la recherche (SPEC-19, ADR-0015).
/// `connecteur` choisit le fournisseur ; les connecteurs connus construisent leurs arguments à partir de
/// `modele` et `effort`. Le connecteur `cli` garde la commande et les arguments à la main (jetons
/// `{consigne}`, `{schema}`, `{mcp}` remplacés à chaque appel), comme avant la v8.
///
/// @req EF-155
/// @adr 15
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigLlm {
    /// `claude`, `agy`, `vibe`, `ollama`, `llama-cpp`, `cli` ou `factice` ; absent : déduit de
    /// `fournisseur` et des arguments (configurations d'avant la v8).
    pub connecteur: Option<String>,
    /// Ancien choix (`cli` ou `factice`), gardé pour les fichiers existants.
    pub fournisseur: String,
    /// Binaire du CLI (vide : celui du connecteur).
    pub commande: String,
    pub arguments: Vec<String>,
    /// Modèle demandé (vide : celui par défaut du fournisseur).
    pub modele: String,
    /// Effort de raisonnement (`low` à `max` selon le fournisseur ; vide : par défaut).
    pub effort: String,
    /// Adresse d'un serveur local (Ollama, llama.cpp) ; vide : celle par défaut.
    pub url: String,
    /// Délai maximum d'un appel, en secondes.
    pub delai: u64,
}

impl ConfigLlm {
    /// Connecteur effectif : celui choisi, sinon `factice` pour `fournisseur = "factice"`, `cli` pour
    /// des arguments modifiés à la main, `claude` sinon.
    pub fn connecteur(&self) -> String {
        if let Some(c) = self.connecteur.as_deref().filter(|c| !c.is_empty()) {
            return c.to_string();
        }
        if self.fournisseur == "factice" {
            "factice".into()
        } else if self.arguments != ConfigLlm::default().arguments
            || !matches!(self.commande.as_str(), "" | "claude")
        {
            "cli".into()
        } else {
            "claude".into()
        }
    }
}

impl Default for ConfigLlm {
    /// LLM par défaut : Claude Code en CLI (`claude`), sans modèle ni effort imposés.
    fn default() -> Self {
        let a = |s: &str| s.to_string();
        Self {
            connecteur: None,
            fournisseur: a("cli"),
            commande: a("claude"),
            modele: String::new(),
            effort: String::new(),
            url: String::new(),
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
    /// Langages indexés par défaut : Rust, TypeScript, JavaScript, Svelte ; rien d'exclu en plus du `.gitignore`.
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
    /// Zone Projet dans `docs/projet`.
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
    /// Doc exigée sur les symboles publics ; sévérités par défaut de chaque diagnostic.
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
