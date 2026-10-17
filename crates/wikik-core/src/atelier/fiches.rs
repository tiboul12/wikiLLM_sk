//! Fiches de technique (SPEC-18) : embarquées dans le binaire, lues depuis `atelier/techniques/`.
//!
//! Le frontmatter décrit les sous-étapes (machine), le corps porte le rôle, les consignes au LLM par
//! sous-étape, les règles et un exemple MyRandonnée (fournisseur factice, tests).
//!
//! @req EF-102
//! @adr 11

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::OnceLock;
use ts_rs::TS;

const SOURCES: &[(&str, &str)] = &[
    (
        "00-clarification.md",
        include_str!("../../../../atelier/techniques/00-clarification.md"),
    ),
    (
        "01-pitch.md",
        include_str!("../../../../atelier/techniques/01-pitch.md"),
    ),
    (
        "02-free-writing.md",
        include_str!("../../../../atelier/techniques/02-free-writing.md"),
    ),
    (
        "03-carte-mentale.md",
        include_str!("../../../../atelier/techniques/03-carte-mentale.md"),
    ),
    (
        "04-analogies.md",
        include_str!("../../../../atelier/techniques/04-analogies.md"),
    ),
    (
        "05-5w1h.md",
        include_str!("../../../../atelier/techniques/05-5w1h.md"),
    ),
    (
        "06-cubing.md",
        include_str!("../../../../atelier/techniques/06-cubing.md"),
    ),
    (
        "07-scamper.md",
        include_str!("../../../../atelier/techniques/07-scamper.md"),
    ),
    (
        "08-storyboard.md",
        include_str!("../../../../atelier/techniques/08-storyboard.md"),
    ),
    (
        "09-bilan.md",
        include_str!("../../../../atelier/techniques/09-bilan.md"),
    ),
    (
        "10-exigences.md",
        include_str!("../../../../atelier/techniques/10-exigences.md"),
    ),
    (
        "11-roadmap.md",
        include_str!("../../../../atelier/techniques/11-roadmap.md"),
    ),
];

/// Valeur qui dépend du mode : Express ou Complet (B6).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Modes {
    pub express: u32,
    pub complet: u32,
}

impl Modes {
    /// Valeur pour le mode donné (`express` ou `complet`).
    pub fn pour(&self, mode: &str) -> u32 {
        if mode == "complet" {
            self.complet
        } else {
            self.express
        }
    }
}

/// Sous-étape d'une technique : `saisie`, `propositions`, `selection` ou `synthese`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SousEtape {
    pub id: String,
    pub genre: String,
    /// Type d'artefact créé pour chaque élément gardé ou saisi.
    #[serde(default)]
    pub cree: Option<String>,
    /// Type d'artefact complété (storyboard, clarification) au lieu d'en créer un.
    #[serde(default)]
    pub complete: Option<String>,
    /// Élément ajouté à un artefact de synthèse (liens de la carte).
    #[serde(default)]
    pub ajoute_a: Option<String>,
    /// Écran dédié, sinon écran générique.
    #[serde(default)]
    pub ecran: Option<String>,
    /// La sous-étape est répétée pour chaque artefact de ce type (ou de la sélection).
    #[serde(default)]
    pub pour_chaque: Option<String>,
    /// Pour une `selection` : type d'artefact proposé au choix.
    #[serde(default)]
    pub parmi: Option<String>,
    #[serde(default)]
    pub nombre: Option<Modes>,
    #[serde(default)]
    pub variantes: Option<Modes>,
    /// Nombre d'éléments à garder conseillé.
    #[serde(default)]
    pub garder: Option<Modes>,
    #[serde(default)]
    pub duree_minutes: Option<Modes>,
    /// Pour une `selection` : présélection (`tous`, ou l'identifiant d'une étape dont reprendre le choix).
    #[serde(default)]
    pub preselection: Option<String>,
    /// Méthode d'analyse (`5w1h`, `cubing`).
    #[serde(default)]
    pub methode: Option<String>,
    /// Le LLM peut proposer un brouillon d'une `saisie`.
    #[serde(default)]
    pub brouillon: bool,
    #[serde(default)]
    #[ts(type = "Record<string, unknown> | null")]
    pub schema: Option<Value>,
    /// Consigne envoyée au LLM (corps de la fiche, `### <id>`).
    #[serde(default)]
    pub consigne: Option<String>,
}

/// Fiche de technique.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Fiche {
    pub id: String,
    pub titre: String,
    pub ordre: u32,
    pub phase: String,
    pub origine: Vec<String>,
    pub optionnelle: bool,
    #[serde(default)]
    pub hors_parcours: bool,
    pub sous_etapes: Vec<SousEtape>,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub regles: Vec<String>,
    /// Exemple MyRandonnée : sous-étape → éléments.
    #[serde(default)]
    #[ts(type = "Record<string, unknown>")]
    pub exemple: Value,
}

impl Fiche {
    /// Sous-étape par identifiant.
    pub fn sous_etape(&self, id: &str) -> Result<&SousEtape> {
        self.sous_etapes
            .iter()
            .find(|s| s.id == id)
            .with_context(|| format!("sous-étape inconnue : {}/{id}", self.id))
    }
}

fn section<'a>(corps: &'a str, titre: &str) -> &'a str {
    let marque = format!("\n## {titre}\n");
    let Some(debut) = corps.find(&marque).map(|i| i + marque.len()) else {
        return "";
    };
    let fin = corps[debut..]
        .find("\n## ")
        .map_or(corps.len(), |i| debut + i);
    corps[debut..fin].trim()
}

/// Analyse une fiche (frontmatter + corps).
pub fn analyser(texte: &str) -> Result<Fiche> {
    let (yaml, corps, _) = crate::projet::separer_frontmatter(texte);
    let yaml = yaml.context("fiche sans frontmatter")?;
    let mut fiche: Fiche =
        serde_yaml_ng::from_str(yaml).context("frontmatter de fiche invalide")?;
    let corps = format!("\n{corps}");
    fiche.role = section(&corps, "Rôle").to_string();
    fiche.regles = section(&corps, "Règles")
        .lines()
        .filter_map(|l| l.strip_prefix("- "))
        .map(str::to_string)
        .collect();
    let consignes = section(&corps, "Consignes");
    let mut par_id: BTreeMap<String, String> = BTreeMap::new();
    for bloc in consignes.split("### ").skip(1) {
        let (id, texte) = bloc.split_once('\n').unwrap_or((bloc, ""));
        par_id.insert(id.trim().to_string(), texte.trim().to_string());
    }
    for se in &mut fiche.sous_etapes {
        se.consigne = par_id.remove(&se.id);
    }
    let exemple = section(&corps, "Exemple");
    if let Some(debut) = exemple.find("```json") {
        let json = &exemple[debut + 7..];
        let fin = json.find("```").unwrap_or(json.len());
        fiche.exemple = serde_json::from_str(&json[..fin]).context("exemple de fiche invalide")?;
    }
    Ok(fiche)
}

/// Toutes les fiches, triées par ordre (la clarification, hors parcours, a l'ordre 0).
pub fn fiches() -> &'static [Fiche] {
    static FICHES: OnceLock<Vec<Fiche>> = OnceLock::new();
    FICHES.get_or_init(|| {
        let mut v: Vec<Fiche> = SOURCES
            .iter()
            .map(|(nom, texte)| analyser(texte).unwrap_or_else(|e| panic!("fiche {nom} : {e:#}")))
            .collect();
        v.sort_by_key(|f| f.ordre);
        v
    })
}

/// Fiche par identifiant.
pub fn fiche(id: &str) -> Result<&'static Fiche> {
    fiches()
        .iter()
        .find(|f| f.id == id)
        .with_context(|| format!("technique inconnue : {id}"))
}

/// Validation minimale d'une valeur contre un schéma JSON (types, `required`, `enum`, bornes de tableau) :
/// assez pour refuser une réponse mal formée du LLM (SPEC-19 R4).
pub fn valider(schema: &Value, valeur: &Value, chemin: &str) -> Result<()> {
    if let Some(valeurs) = schema.get("enum").and_then(Value::as_array)
        && !valeurs.contains(valeur)
    {
        bail!("{chemin} : valeur hors liste ({valeur})");
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("object") => {
            let Some(objet) = valeur.as_object() else {
                bail!("{chemin} : objet attendu")
            };
            for r in schema
                .get("required")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                if !objet.contains_key(r) {
                    bail!("{chemin} : champ « {r} » manquant");
                }
            }
            if let Some(props) = schema.get("properties").and_then(Value::as_object) {
                for (k, s) in props {
                    if let Some(v) = objet.get(k) {
                        valider(s, v, &format!("{chemin}.{k}"))?;
                    }
                }
            }
        }
        Some("array") => {
            let Some(t) = valeur.as_array() else {
                bail!("{chemin} : tableau attendu")
            };
            if let Some(min) = schema.get("minItems").and_then(Value::as_u64)
                && (t.len() as u64) < min
            {
                bail!(
                    "{chemin} : au moins {min} éléments attendus, {} reçus",
                    t.len()
                );
            }
            if let Some(max) = schema.get("maxItems").and_then(Value::as_u64)
                && (t.len() as u64) > max
            {
                bail!(
                    "{chemin} : au plus {max} éléments attendus, {} reçus",
                    t.len()
                );
            }
            if let Some(items) = schema.get("items") {
                for (i, v) in t.iter().enumerate() {
                    valider(items, v, &format!("{chemin}[{i}]"))?;
                }
            }
        }
        Some("string") if !valeur.is_string() => bail!("{chemin} : texte attendu"),
        Some("integer") if !valeur.is_i64() && !valeur.is_u64() => {
            bail!("{chemin} : entier attendu")
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn douze_fiches_et_exemples_conformes() {
        let f = fiches();
        assert_eq!(f.len(), 12);
        assert_eq!(f.iter().filter(|x| !x.hors_parcours).count(), 11);
        for fiche in f {
            assert!(!fiche.role.is_empty(), "{} : rôle", fiche.id);
            for se in &fiche.sous_etapes {
                if se.genre == "propositions" {
                    assert!(se.consigne.is_some(), "{}/{} : consigne", fiche.id, se.id);
                    let schema = se
                        .schema
                        .as_ref()
                        .unwrap_or_else(|| panic!("{}/{} : schéma", fiche.id, se.id));
                    if let Some(ex) = fiche.exemple.get(&se.id) {
                        // L'exemple est une liste d'éléments ; le schéma décrit `{ elements: [...] }`.
                        let mut s = schema.clone();
                        s["properties"]["elements"]
                            .as_object_mut()
                            .unwrap()
                            .remove("minItems");
                        valider(
                            &s,
                            &json!({ "elements": ex }),
                            &format!("{}/{}", fiche.id, se.id),
                        )
                        .unwrap();
                    }
                }
            }
        }
        assert_eq!(
            fiche("scamper")
                .unwrap()
                .sous_etape("leviers")
                .unwrap()
                .variantes
                .unwrap()
                .pour("express"),
            7
        );
    }

    #[test]
    fn validation() {
        let s = json!({"type":"object","properties":{"elements":{"type":"array","minItems":2,"items":{"type":"object","properties":{"p":{"type":"string","enum":["a","b"]}},"required":["p"]}}},"required":["elements"]});
        assert!(valider(&s, &json!({"elements":[{"p":"a"},{"p":"b"}]}), "r").is_ok());
        assert!(valider(&s, &json!({"elements":[{"p":"a"}]}), "r").is_err());
        assert!(valider(&s, &json!({"elements":[{"p":"z"},{"p":"a"}]}), "r").is_err());
        assert!(valider(&s, &json!({"elements":[{"q":"a"},{"p":"a"}]}), "r").is_err());
    }
}
