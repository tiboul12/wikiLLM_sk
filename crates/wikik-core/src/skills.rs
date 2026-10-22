//! Consignes des LLM en skills du projet (ADR-0017, EF-156) : `.claude/skills/<nom>/SKILL.md`, au format
//! Claude Code. Ce sont les textes réellement envoyés : wikik relit le fichier à chaque appel ; tant qu'il
//! n'existe pas, le texte d'origine embarqué dans le binaire sert. Étant dans `.claude/skills/`, ils sont
//! aussi ceux de l'agent de développement qui travaille dans le projet.
//!
//! @req EF-156
//! @adr 17

use crate::config::Config;
use anyhow::{Result, bail};
use serde::Serialize;
use std::path::{Path, PathBuf};
use ts_rs::TS;

/// Un skill livré par wikik.
pub struct Definition {
    pub nom: &'static str,
    /// Nom affiché dans les Paramètres.
    pub titre: &'static str,
    /// Quand wikik s'en sert, en une phrase.
    pub role: &'static str,
    /// Groupe des Paramètres : `agent`, `llm` ou `structure`.
    pub groupe: &'static str,
    pub origine: &'static str,
}

/// Skills livrés, dans l'ordre des Paramètres.
pub const SKILLS: &[Definition] = &[
    Definition {
        nom: "wikik",
        titre: "Documenter le code pour wikik",
        role: "Le skill de l'agent de développement : interroger l'index avant d'ouvrir le code, écrire les blocs `///` qui rendent le code lisible par wikik, poser `@req` et `@adr`, passer `wikik check` avant chaque commit.",
        groupe: "agent",
        origine: include_str!("../../../skill/SKILL.md"),
    },
    Definition {
        nom: "wikik-traiter-reception",
        titre: "Traiter le cahier de réception",
        role: "Envoyé à l'agent par « Traiter avec l'agent » (cahier de réception) : plan par lots validé, puis correction, vérification, commit et enregistrement de chaque remarque.",
        groupe: "agent",
        origine: include_str!("../skills/wikik-traiter-reception.md"),
    },
    Definition {
        nom: "wikik-lecture",
        titre: "Lire le projet avec wikik",
        role: "Préambule de toutes les demandes que wikik fait à son LLM (Design thinking, réception, recherche) et instructions de son serveur MCP : comment lire le projet sans ouvrir les fichiers.",
        groupe: "llm",
        origine: include_str!("../skills/wikik-lecture.md"),
    },
    Definition {
        nom: "wikik-clarification",
        titre: "Clarifier un sujet",
        role: "Consigne de l'écran Clarification : juger si la question porte sur le sujet, puis proposer des pistes sourcées.",
        groupe: "llm",
        origine: include_str!("../skills/wikik-clarification.md"),
    },
    Definition {
        nom: "wikik-structure-besoin",
        titre: "Structure des besoins",
        role: "Comment sont rédigés les besoins (B-xxx) : donné au LLM quand il en produit, et à l'agent.",
        groupe: "structure",
        origine: include_str!("../skills/wikik-structure-besoin.md"),
    },
    Definition {
        nom: "wikik-structure-exigence",
        titre: "Structure des exigences",
        role: "Comment sont rédigées les exigences (EF-xxx, ENF-xxx) et leur place dans la cascade jusqu'aux tests.",
        groupe: "structure",
        origine: include_str!("../skills/wikik-structure-exigence.md"),
    },
    Definition {
        nom: "wikik-structure-test",
        titre: "Structure des tests",
        role: "Comment sont rédigés les tests d'exigence (TEST-xxx) et leur ticket de validation.",
        groupe: "structure",
        origine: include_str!("../skills/wikik-structure-test.md"),
    },
    Definition {
        nom: "wikik-structure-adr",
        titre: "Structure des décisions (ADR)",
        role: "Comment sont rédigées les décisions d'architecture.",
        groupe: "structure",
        origine: include_str!("../skills/wikik-structure-adr.md"),
    },
    Definition {
        nom: "wikik-structure-spec",
        titre: "Structure des specs",
        role: "Comment sont rédigées les spécifications.",
        groupe: "structure",
        origine: include_str!("../skills/wikik-structure-spec.md"),
    },
    Definition {
        nom: "wikik-structure-bdr",
        titre: "Structure des BDR",
        role: "Comment sont rédigés les deux BDR d'une version majeure.",
        groupe: "structure",
        origine: include_str!("../skills/wikik-structure-bdr.md"),
    },
    Definition {
        nom: "wikik-structure-cahier-des-charges",
        titre: "Structure des cahiers des charges",
        role: "Comment est rédigé le cahier des charges d'une version majeure.",
        groupe: "structure",
        origine: include_str!("../skills/wikik-structure-cahier-des-charges.md"),
    },
];

/// Skill de structure d'un type de document (`besoin`, `exigence-nf`, `bdr-analyse`…), s'il existe.
pub fn structure_de(genre: &str) -> Option<&'static str> {
    let g = match genre {
        "exigence-nf" => "exigence",
        "bdr-analyse" | "bdr-definition" => "bdr",
        g => g,
    };
    SKILLS
        .iter()
        .find(|d| d.nom.strip_prefix("wikik-structure-") == Some(g))
        .map(|d| d.nom)
}

fn definition(nom: &str) -> Result<&'static Definition> {
    match SKILLS.iter().find(|d| d.nom == nom) {
        Some(d) => Ok(d),
        None => bail!("skill inconnu : {nom}"),
    }
}

/// Fichier du skill dans le projet.
pub fn chemin(racine: &Path, nom: &str) -> PathBuf {
    racine
        .join(".claude")
        .join("skills")
        .join(nom)
        .join("SKILL.md")
}

/// Texte du skill : le fichier du projet, sinon une consigne d'avant la v8 gardée dans `wikik.toml`
/// (`[ia] consigne_lecture`, `consigne_traitement`), sinon le texte d'origine.
pub fn texte(racine: &Path, config: &Config, nom: &str) -> String {
    if let Ok(t) = std::fs::read_to_string(chemin(racine, nom))
        && !t.trim().is_empty()
    {
        return t;
    }
    let ancien = config::ancienne_consigne(config, nom);
    match (ancien, definition(nom)) {
        (Some(a), Ok(d)) => format!("{}\n{a}\n", entete(d.origine)),
        (None, Ok(d)) => d.origine.to_string(),
        (_, Err(_)) => String::new(),
    }
}

/// Corps du skill, sans son frontmatter : ce qui part dans une demande au LLM.
pub fn corps(racine: &Path, config: &Config, nom: &str) -> String {
    let t = texte(racine, config, nom);
    sans_entete(&t).trim().to_string()
}

fn entete(t: &str) -> &str {
    let n = t.len() - sans_entete(t).len();
    &t[..n]
}

fn sans_entete(t: &str) -> &str {
    if let Some(reste) = t.strip_prefix("---\n")
        && let Some(fin) = reste.find("\n---\n")
    {
        return &reste[fin + 5..];
    }
    t
}

mod config {
    use crate::config::{Config, ConfigIa};

    /// Consigne d'avant la v8 réglée dans `wikik.toml`, si elle diffère du défaut.
    pub(super) fn ancienne_consigne(c: &Config, nom: &str) -> Option<String> {
        let d = ConfigIa::default();
        let (v, defaut) = match nom {
            "wikik-lecture" => (&c.ia.consigne_lecture, &d.consigne_lecture),
            "wikik-traiter-reception" => (&c.ia.consigne_traitement, &d.consigne_traitement),
            _ => return None,
        };
        (v != defaut).then(|| v.clone())
    }
}

/// Un skill tel que les Paramètres l'affichent.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Skill {
    pub nom: String,
    pub titre: String,
    pub role: String,
    pub groupe: String,
    /// Chemin du fichier dans le projet.
    pub chemin: String,
    pub texte: String,
    /// Le fichier existe dans le projet (sinon le texte d'origine sert).
    pub installe: bool,
    /// Le texte diffère de celui d'origine.
    pub modifie: bool,
}

/// Les skills livrés, avec leur texte actuel.
pub fn lister(racine: &Path, config: &Config) -> Vec<Skill> {
    SKILLS
        .iter()
        .map(|d| {
            let t = texte(racine, config, d.nom);
            Skill {
                nom: d.nom.into(),
                titre: d.titre.into(),
                role: d.role.into(),
                groupe: d.groupe.into(),
                chemin: format!(".claude/skills/{}/SKILL.md", d.nom),
                installe: chemin(racine, d.nom).is_file(),
                modifie: t.trim() != d.origine.trim(),
                texte: t,
            }
        })
        .collect()
}

/// Écrit le skill dans le projet. Le texte doit garder un frontmatter `name` et `description` (format
/// skill) ; un texte vide est refusé.
///
/// @pitfall Le dossier du skill peut être un lien symbolique (`.claude/skills/wikik` → `skill/` dans
/// wikiK lui-même) : on écrit à travers le lien, sans le remplacer.
pub fn ecrire(racine: &Path, nom: &str, texte: &str) -> Result<()> {
    definition(nom)?;
    let t = texte.trim();
    if t.is_empty() {
        bail!("skill vide");
    }
    if !t.starts_with("---") || !t.contains("\nname:") || !t.contains("\ndescription:") {
        bail!("un skill commence par un frontmatter avec `name` et `description`");
    }
    let f = chemin(racine, nom);
    std::fs::create_dir_all(f.parent().expect("dossier du skill"))?;
    std::fs::write(&f, format!("{t}\n"))?;
    Ok(())
}

/// Installe le skill avec son texte actuel s'il n'est pas encore dans le projet (l'agent le trouve alors).
pub fn installer(racine: &Path, config: &Config, nom: &str) -> Result<PathBuf> {
    let f = chemin(racine, nom);
    if !f.is_file() {
        ecrire(racine, nom, &texte(racine, config, nom))?;
    }
    Ok(f)
}

/// Écrit dans son skill une consigne d'avant la v8 gardée dans `wikik.toml`, avant qu'elle n'en soit
/// retirée ; rien si elle vaut le défaut ou si le skill existe déjà.
pub fn installer_si_ancienne(racine: &Path, config: &Config, nom: &str) -> Result<()> {
    if config::ancienne_consigne(config, nom).is_some() && !chemin(racine, nom).is_file() {
        ecrire(racine, nom, &texte(racine, config, nom))?;
    }
    Ok(())
}

/// Revient au texte d'origine : le fichier du projet est supprimé (le dossier aussi s'il est vide).
pub fn retablir(racine: &Path, nom: &str) -> Result<()> {
    definition(nom)?;
    let f = chemin(racine, nom);
    if f.is_file() {
        std::fs::remove_file(&f)?;
        let _ = std::fs::remove_dir(f.parent().expect("dossier du skill"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origine_puis_fichier_puis_retour() {
        let d = tempfile::tempdir().unwrap();
        let c = Config::default();
        let t = corps(d.path(), &c, "wikik-lecture");
        assert!(t.starts_with("# Lire le projet avec wikik"), "{t}");
        assert!(ecrire(d.path(), "wikik-lecture", "sans frontmatter").is_err());
        ecrire(
            d.path(),
            "wikik-lecture",
            "---\nname: wikik-lecture\ndescription: x\n---\n\nLis tout.",
        )
        .unwrap();
        assert_eq!(corps(d.path(), &c, "wikik-lecture"), "Lis tout.");
        assert!(
            lister(d.path(), &c)
                .iter()
                .any(|s| s.nom == "wikik-lecture" && s.modifie && s.installe)
        );
        retablir(d.path(), "wikik-lecture").unwrap();
        assert!(!chemin(d.path(), "wikik-lecture").exists());
        assert_eq!(
            structure_de("exigence-nf"),
            Some("wikik-structure-exigence")
        );
    }

    /// Une consigne réglée avant la v8 dans `wikik.toml` reste celle qui sert.
    #[test]
    fn ancienne_consigne_reprise() {
        let d = tempfile::tempdir().unwrap();
        let mut c = Config::default();
        c.ia.consigne_traitement = "Commence par un plan.".into();
        let t = texte(d.path(), &c, "wikik-traiter-reception");
        assert!(
            t.starts_with("---\nname: wikik-traiter-reception")
                && t.trim_end().ends_with("Commence par un plan."),
            "{t}"
        );
    }
}
