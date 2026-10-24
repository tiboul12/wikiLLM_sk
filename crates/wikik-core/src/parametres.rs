//! Paramètres du projet réglés à l'écran (EF-147) : LLM de l'atelier et de la réception, consignes des
//! LLM, agent de développement. Ils vivent dans `wikik.toml`, réécrit en gardant ses commentaires ; une
//! valeur égale au défaut est retirée du fichier, qui reste court.
//!
//! @req EF-147
//! @adr 13

use crate::config::{Config, ConfigIa, ConfigLlm};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table, value};
use ts_rs::TS;

/// Paramètres affichés et modifiables dans la section Paramètres.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Parametres {
    /// `cli` ou `factice`.
    pub fournisseur: String,
    pub commande: String,
    pub arguments: Vec<String>,
    /// Délai maximum d'un appel, en secondes.
    pub delai: u64,
    pub consigne_lecture: String,
    pub consigne_traitement: String,
    pub agent: String,
}

impl Parametres {
    /// Paramètres d'une configuration chargée.
    pub fn de(config: &Config) -> Parametres {
        let l = &config.atelier.llm;
        Parametres {
            fournisseur: l.fournisseur.clone(),
            commande: l.commande.clone(),
            arguments: l.arguments.clone(),
            delai: l.delai,
            consigne_lecture: config.ia.consigne_lecture.clone(),
            consigne_traitement: config.ia.consigne_traitement.clone(),
            agent: config.ia.agent.clone(),
        }
    }
}

/// Écrit les paramètres dans `<racine>/wikik.toml` (créé au besoin) et renvoie la configuration relue.
/// Le fichier n'est remplacé que si la nouvelle configuration se charge.
pub fn ecrire(racine: &Path, p: &Parametres) -> Result<Config> {
    if !matches!(p.fournisseur.as_str(), "cli" | "factice") {
        bail!("fournisseur inconnu : {} (cli ou factice)", p.fournisseur);
    }
    if p.fournisseur == "cli" && p.commande.trim().is_empty() {
        bail!("la commande du LLM est vide");
    }
    let chemin = racine.join("wikik.toml");
    let ancien = std::fs::read_to_string(&chemin).unwrap_or_default();
    let mut doc: DocumentMut = ancien.parse().context("wikik.toml illisible")?;
    let (dl, di) = (ConfigLlm::default(), ConfigIa::default());
    let llm = table(&mut doc, &["atelier", "llm"]);
    regler(llm, "fournisseur", &p.fournisseur, &dl.fournisseur);
    regler(llm, "commande", p.commande.trim(), &dl.commande);
    if p.arguments == dl.arguments {
        llm.remove("arguments");
    } else {
        llm["arguments"] = value(p.arguments.iter().collect::<toml_edit::Array>());
    }
    if p.delai == dl.delai {
        llm.remove("delai");
    } else {
        llm["delai"] = value(i64::try_from(p.delai).unwrap_or(i64::MAX));
    }
    let ia = table(&mut doc, &["ia"]);
    regler(
        ia,
        "consigne_lecture",
        p.consigne_lecture.trim(),
        &di.consigne_lecture,
    );
    regler(
        ia,
        "consigne_traitement",
        p.consigne_traitement.trim(),
        &di.consigne_traitement,
    );
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
        ecrire(d.path(), &p).unwrap();
        assert_eq!(
            std::fs::read_to_string(d.path().join("wikik.toml")).unwrap(),
            "# Mon projet\n[serveur]\nreseau = true # depuis le portable\n"
        );
        p.consigne_traitement = "Commence par un plan.".into();
        p.delai = 60;
        let c = ecrire(d.path(), &p).unwrap();
        assert_eq!(
            (c.ia.consigne_traitement.as_str(), c.atelier.llm.delai),
            ("Commence par un plan.", 60)
        );
        let t = std::fs::read_to_string(d.path().join("wikik.toml")).unwrap();
        assert!(
            t.starts_with("# Mon projet") && t.contains("# depuis le portable"),
            "{t}"
        );
        p.fournisseur = "autre".into();
        assert!(ecrire(d.path(), &p).is_err());
    }
}
