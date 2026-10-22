//! Paramètres du projet réglés à l'écran (EF-147, EF-155) : connexion au LLM de l'atelier, de la réception
//! et de la recherche (connecteur, modèle, effort, adresse) et agent de développement. Ils vivent dans
//! `wikik.toml`, réécrit en gardant ses commentaires ; une valeur égale au défaut est retirée du fichier,
//! qui reste court. Les consignes des LLM sont des skills (ADR-0017, module `skills`).
//!
//! @req EF-147, EF-155
//! @adr 13
//! @adr 15

use crate::config::{Config, ConfigIa, ConfigLlm};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table, value};
use ts_rs::TS;

/// Paramètres affichés et modifiables dans la section Paramètres.
/// @relu 2026-10-22 8b5561b1
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Parametres {
    /// `claude`, `agy`, `vibe`, `ollama`, `llama-cpp`, `cli` ou `factice` (ADR-0015).
    pub connecteur: String,
    /// Binaire du CLI (vide : celui du connecteur).
    pub commande: String,
    /// Arguments du connecteur `cli` (commande libre).
    pub arguments: Vec<String>,
    pub modele: String,
    pub effort: String,
    /// Adresse du serveur local (Ollama, llama.cpp) ; vide : celle par défaut.
    pub url: String,
    /// Délai maximum d'un appel, en secondes.
    pub delai: u64,
    pub agent: String,
}

impl Parametres {
    /// Paramètres d'une configuration chargée.
    /// @relu 2026-10-22 b24b0c40
    pub fn de(config: &Config) -> Parametres {
        let l = &config.atelier.llm;
        Parametres {
            connecteur: l.connecteur(),
            commande: l.commande.clone(),
            arguments: l.arguments.clone(),
            modele: l.modele.clone(),
            effort: l.effort.clone(),
            url: l.url.clone(),
            delai: l.delai,
            agent: config.ia.agent.clone(),
        }
    }
}

/// Écrit les paramètres dans `<racine>/wikik.toml` (créé au besoin) et renvoie la configuration relue.
/// Le fichier n'est remplacé que si la nouvelle configuration se charge. Une consigne d'avant la v8
/// (`[ia] consigne_lecture`, `consigne_traitement`) passe dans son skill et quitte le fichier.
pub fn ecrire(racine: &Path, p: &Parametres) -> Result<Config> {
    let connus = crate::atelier::connecteurs::connecteurs();
    if !connus.iter().any(|c| c.id == p.connecteur) {
        bail!("connecteur inconnu : {}", p.connecteur);
    }
    if p.connecteur == "cli" && p.commande.trim().is_empty() {
        bail!("la commande du LLM est vide");
    }
    let chemin = racine.join("wikik.toml");
    let ancien = std::fs::read_to_string(&chemin).unwrap_or_default();
    let avant: Config = toml::from_str(&ancien).unwrap_or_default();
    for nom in ["wikik-lecture", "wikik-traiter-reception"] {
        crate::skills::installer_si_ancienne(racine, &avant, nom)?;
    }
    let mut doc: DocumentMut = ancien.parse().context("wikik.toml illisible")?;
    let (dl, di) = (ConfigLlm::default(), ConfigIa::default());
    let llm = table(&mut doc, &["atelier", "llm"]);
    llm.remove("fournisseur");
    // Claude Code est le connecteur par défaut : il n'est pas écrit.
    regler(llm, "connecteur", &p.connecteur, "claude");
    regler(llm, "commande", p.commande.trim(), &dl.commande);
    if p.connecteur != "cli" || p.arguments == dl.arguments {
        llm.remove("arguments");
    } else {
        llm["arguments"] = value(p.arguments.iter().collect::<toml_edit::Array>());
    }
    regler(llm, "modele", p.modele.trim(), "");
    regler(llm, "effort", p.effort.trim(), "");
    regler(llm, "url", p.url.trim(), "");
    if p.delai == dl.delai {
        llm.remove("delai");
    } else {
        llm["delai"] = value(i64::try_from(p.delai).unwrap_or(i64::MAX));
    }
    let ia = table(&mut doc, &["ia"]);
    ia.remove("consigne_lecture");
    ia.remove("consigne_traitement");
    regler(ia, "agent", p.agent.trim(), &di.agent);
    nettoyer(&mut doc);
    let texte = doc.to_string();
    let config: Config = toml::from_str(&texte).context("paramètres refusés")?;
    std::fs::write(&chemin, texte)?;
    Ok(config)
}

/// Table `[a.b]` du document, créée au besoin (implicite si elle n'a que des sous-tables).
fn table<'d>(doc: &'d mut DocumentMut, chemin: &[&str]) -> &'d mut Table {
    let mut t = doc.as_table_mut();
    for nom in chemin {
        let entree = t.entry(nom).or_insert_with(|| {
            let mut n = Table::new();
            n.set_implicit(true);
            Item::Table(n)
        });
        t = entree.as_table_mut().expect("section de wikik.toml");
    }
    t
}

/// Pose une clé, ou la retire quand elle vaut le défaut.
fn regler(t: &mut Table, cle: &str, v: &str, defaut: &str) {
    if v == defaut {
        t.remove(cle);
    } else {
        t[cle] = value(v);
    }
}

/// Retire les sections devenues vides par `regler` (`[ia]`, `[atelier.llm]`, `[atelier]`).
fn nettoyer(doc: &mut DocumentMut) {
    let vide = |i: Option<&Item>| i.and_then(Item::as_table).is_some_and(Table::is_empty);
    if vide(doc.get("ia")) {
        doc.remove("ia");
    }
    if let Some(a) = doc.get_mut("atelier").and_then(Item::as_table_mut)
        && vide(a.get("llm"))
    {
        a.remove("llm");
    }
    if vide(doc.get("atelier")) {
        doc.remove("atelier");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les commentaires restent, une valeur par défaut n'est pas écrite, une valeur changée l'est.
    #[test]
    fn ecriture_minimale_et_commentee() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("wikik.toml"),
            "# Mon projet\n[serveur]\nreseau = true # depuis le portable\n",
        )
        .unwrap();
        let mut p = Parametres::de(&Config::charger(d.path()).unwrap());
        assert_eq!(p.connecteur, "claude");
        ecrire(d.path(), &p).unwrap();
        assert_eq!(
            std::fs::read_to_string(d.path().join("wikik.toml")).unwrap(),
            "# Mon projet\n[serveur]\nreseau = true # depuis le portable\n"
        );
        p.modele = "sonnet".into();
        p.effort = "medium".into();
        p.delai = 60;
        let c = ecrire(d.path(), &p).unwrap();
        assert_eq!(
            (c.atelier.llm.modele.as_str(), c.atelier.llm.delai),
            ("sonnet", 60)
        );
        let t = std::fs::read_to_string(d.path().join("wikik.toml")).unwrap();
        assert!(
            t.starts_with("# Mon projet")
                && t.contains("# depuis le portable")
                && !t.contains("connecteur"),
            "{t}"
        );
        p.connecteur = "autre".into();
        assert!(ecrire(d.path(), &p).is_err());
    }

    /// Une consigne d'avant la v8 passe dans son skill et quitte wikik.toml.
    #[test]
    fn consigne_migree_en_skill() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("wikik.toml"),
            "[ia]\nconsigne_traitement = \"Commence par un plan.\"\n",
        )
        .unwrap();
        let p = Parametres::de(&Config::charger(d.path()).unwrap());
        ecrire(d.path(), &p).unwrap();
        let t = std::fs::read_to_string(d.path().join("wikik.toml")).unwrap();
        assert!(!t.contains("consigne_traitement"), "{t}");
        let skill =
            std::fs::read_to_string(crate::skills::chemin(d.path(), "wikik-traiter-reception"))
                .unwrap();
        assert!(skill.contains("Commence par un plan."), "{skill}");
    }
}
