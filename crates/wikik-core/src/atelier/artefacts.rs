//! Artefacts de l'atelier : un fichier Markdown par identifiant, sous `docs/atelier/` (ou `docs/projet/`
//! pour ce qui part vers le développement), avec la traçabilité dans `sources` (SPEC-17).
//!
//! @req EF-105, EF-107
//! @adr 10

use crate::config::Config;
use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// Zone du dépôt où vit un type d'artefact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Atelier,
    Projet,
}

/// Description d'un type d'artefact : préfixe d'identifiant, sous-dossier, fichier unique ou non.
#[derive(Debug, Clone, Copy)]
pub struct TypeArtefact {
    pub nom: &'static str,
    pub prefixe: &'static str,
    pub dossier: &'static str,
    pub zone: Zone,
    /// Fichier unique (`pitch.md`, `carte.md`, `parcours.md`) : l'identifiant est le préfixe seul.
    pub fichier_unique: Option<&'static str>,
    pub chiffres: usize,
}

const fn t(
    nom: &'static str,
    prefixe: &'static str,
    dossier: &'static str,
    zone: Zone,
) -> TypeArtefact {
    TypeArtefact {
        nom,
        prefixe,
        dossier,
        zone,
        fichier_unique: None,
        chiffres: 3,
    }
}

const fn unique(nom: &'static str, prefixe: &'static str, fichier: &'static str) -> TypeArtefact {
    TypeArtefact {
        nom,
        prefixe,
        dossier: "",
        zone: Zone::Atelier,
        fichier_unique: Some(fichier),
        chiffres: 0,
    }
}

/// Tous les types d'artefacts (SPEC-17).
pub const TYPES: &[TypeArtefact] = &[
    unique("pitch", "PITCH", "pitch.md"),
    unique("carte", "CARTE", "carte.md"),
    unique("parcours", "PARCOURS", "parcours.md"),
    t("axe", "AXE-", "axes", Zone::Atelier),
    t("ecrit", "FW-", "ecrits", Zone::Atelier),
    t("unite", "US-", "unites", Zone::Atelier),
    t("probleme", "PB-", "problemes", Zone::Atelier),
    t("analogie", "ANL-", "analogies", Zone::Atelier),
    t("idee", "IDEE-", "idees", Zone::Atelier),
    t("analyse", "ANA-", "analyses", Zone::Atelier),
    t("scamper", "SCA-", "scamper", Zone::Atelier),
    t("storyboard", "SB-", "storyboards", Zone::Atelier),
    t("clarification", "CLAR-", "clarifications", Zone::Atelier),
    t("besoin", "B-", "besoins", Zone::Projet),
    t("exigence", "EF-", "exigences", Zone::Projet),
];

/// Type d'artefact par son nom.
pub fn type_de(nom: &str) -> Result<&'static TypeArtefact> {
    TYPES
        .iter()
        .find(|t| t.nom == nom)
        .with_context(|| format!("type d'artefact inconnu : {nom}"))
}

fn dossier_zone(racine: &Path, config: &Config, zone: Zone) -> PathBuf {
    racine.join(match zone {
        Zone::Atelier => config.atelier.dossier.trim_end_matches('/'),
        Zone::Projet => config.projet.dossier.trim_end_matches('/'),
    })
}

/// Chemin du fichier d'un artefact, relatif à la racine du projet.
pub fn chemin_relatif(config: &Config, ty: &TypeArtefact, id: &str) -> String {
    let base = match ty.zone {
        Zone::Atelier => config.atelier.dossier.trim_end_matches('/'),
        Zone::Projet => config.projet.dossier.trim_end_matches('/'),
    };
    match ty.fichier_unique {
        Some(f) => format!("{base}/{f}"),
        None => format!("{base}/{}/{id}.md", ty.dossier),
    }
}

/// Prochain identifiant libre : plus grand numéro trouvé dans les noms de fichiers et les `id:`, plus un.
/// Un numéro n'est jamais réutilisé, même si l'artefact a été écarté (SPEC-17 R2).
pub fn prochain_id(racine: &Path, config: &Config, ty: &TypeArtefact) -> String {
    if ty.fichier_unique.is_some() {
        return ty.prefixe.to_string();
    }
    let mut max = 0u32;
    if let Ok(entrees) = std::fs::read_dir(dossier_zone(racine, config, ty.zone).join(ty.dossier)) {
        for e in entrees.flatten() {
            let nom = e.file_name().to_string_lossy().to_string();
            let texte = std::fs::read_to_string(e.path()).unwrap_or_default();
            let ids = std::iter::once(nom.as_str()).chain(
                texte
                    .lines()
                    .filter_map(|l| l.strip_prefix("id:"))
                    .map(|v| v.trim().trim_matches('"')),
            );
            for candidat in ids {
                if let Some(reste) = candidat.strip_prefix(ty.prefixe) {
                    let chiffres: String =
                        reste.chars().take_while(|c| c.is_ascii_digit()).collect();
                    max = max.max(chiffres.parse().unwrap_or(0));
                }
            }
        }
    }
    format!("{}{:0w$}", ty.prefixe, max + 1, w = ty.chiffres)
}

/// Frontmatter dans un ordre lisible : `id`, `type`, `titre`, `statut` d'abord, puis le reste trié.
/// Les valeurs sont écrites en JSON, qui est du YAML valide.
pub fn formater(frontmatter: &Map<String, Value>, corps: &str) -> String {
    let mut s = String::from("---\n");
    let premiers = ["id", "type", "titre", "statut"];
    for k in premiers {
        if let Some(v) = frontmatter.get(k) {
            s.push_str(&format!("{k}: {v}\n"));
        }
    }
    for (k, v) in frontmatter
        .iter()
        .filter(|(k, _)| !premiers.contains(&k.as_str()))
    {
        s.push_str(&format!("{k}: {v}\n"));
    }
    s.push_str("---\n\n");
    s.push_str(corps.trim());
    s.push('\n');
    s
}

/// Crée un artefact au prochain identifiant libre ; renvoie (identifiant, chemin relatif).
pub fn creer(
    racine: &Path,
    config: &Config,
    type_nom: &str,
    mut frontmatter: Map<String, Value>,
    corps: &str,
) -> Result<(String, String)> {
    let ty = type_de(type_nom)?;
    let id = prochain_id(racine, config, ty);
    frontmatter.insert("id".into(), Value::String(id.clone()));
    frontmatter.insert("type".into(), Value::String(ty.nom.into()));
    let relatif = chemin_relatif(config, ty, &id);
    let chemin = racine.join(&relatif);
    if ty.fichier_unique.is_none() && chemin.exists() {
        bail!("{relatif} existe déjà");
    }
    std::fs::create_dir_all(chemin.parent().unwrap())?;
    std::fs::write(&chemin, formater(&frontmatter, corps))?;
    Ok((id, relatif))
}

/// Lit un artefact (frontmatter en JSON, corps) à partir de son chemin relatif.
pub fn lire(racine: &Path, relatif: &str) -> Result<(Map<String, Value>, String)> {
    let texte = std::fs::read_to_string(racine.join(relatif))
        .with_context(|| format!("lecture de {relatif}"))?;
    let (yaml, corps, _) = crate::projet::separer_frontmatter(&texte);
    let fm = match yaml.map(serde_yaml_ng::from_str::<Value>) {
        Some(Ok(Value::Object(m))) => m,
        _ => Map::new(),
    };
    Ok((fm, corps.to_string()))
}

/// Réécrit un artefact existant (frontmatter et corps), en gardant son identifiant.
pub fn reecrire(
    racine: &Path,
    relatif: &str,
    frontmatter: &Map<String, Value>,
    corps: &str,
) -> Result<()> {
    std::fs::write(racine.join(relatif), formater(frontmatter, corps))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identifiants_et_relecture() {
        let d = tempfile::tempdir().unwrap();
        let c = Config::default();
        let fm = |t: &str| {
            json!({"titre": t, "statut": "retenu", "sources": ["US-001"]})
                .as_object()
                .unwrap()
                .clone()
        };
        let (a, ca) = creer(d.path(), &c, "idee", fm("Alerte"), "# Alerte").unwrap();
        let (b, _) = creer(d.path(), &c, "idee", fm("Retour"), "# Retour").unwrap();
        assert_eq!(
            (a.as_str(), b.as_str(), ca.as_str()),
            ("IDEE-001", "IDEE-002", "docs/atelier/idees/IDEE-001.md")
        );
        let (p, cp) = creer(d.path(), &c, "pitch", fm("MyRandonnée"), "# Pitch").unwrap();
        assert_eq!(
            (p.as_str(), cp.as_str()),
            ("PITCH", "docs/atelier/pitch.md")
        );
        let (lu, corps) = lire(d.path(), &ca).unwrap();
        assert_eq!(
            (lu["id"].as_str(), lu["sources"][0].as_str(), corps.trim()),
            (Some("IDEE-001"), Some("US-001"), "# Alerte")
        );
        let (e, ce) = creer(d.path(), &c, "exigence", fm("X"), "").unwrap();
        assert_eq!(
            (e.as_str(), ce.as_str()),
            ("EF-001", "docs/projet/exigences/EF-001.md")
        );
    }
}
