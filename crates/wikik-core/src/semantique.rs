//! Recherche par le sens (EF-173, ADR-0020), sur le modèle de grepai : chaque morceau de l'index (bloc de doc d'un
//! symbole, document) reçoit un vecteur calculé par le modèle d'embeddings d'Ollama ; une question est ramenée à
//! son vecteur et comparée à tous par similarité cosinus.
//!
//! Les vecteurs vivent dans `.wikik/semantique.sqlite`, non versionné et reconstructible (ADR-0003) ; l'empreinte
//! du texte et du modèle dit lesquels recalculer. Les appels au modèle se font hors du verrou de l'index :
//! [`morceaux`] lit l'index, [`chercher`] calcule et compare.
//!
//! @req EF-173
//! @adr 20

use crate::config::Config;
use crate::index::{DOSSIER, Index};
use anyhow::{Context, Result, bail};
use rusqlite::{Connection, params};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use ts_rs::TS;

/// Longueur maximale du texte d'un morceau, en caractères : au-delà, le modèle n'en lit de toute façon pas plus.
const LONGUEUR: usize = 2000;
/// Morceaux envoyés au modèle par appel.
const LOT: usize = 32;

/// Un morceau cherchable : une ligne de la table `recherche` de l'index.
#[derive(Debug, Clone)]
pub struct Morceau {
    /// Identifiant du symbole ou du document.
    pub cle: String,
    /// `symbole` ou le genre du document.
    pub genre: String,
    pub titre: String,
    /// Texte envoyé au modèle : titre puis texte, tronqué.
    pub texte: String,
}

/// Un résultat de la recherche par le sens, du plus proche au moins proche.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Trouve {
    pub cle: String,
    pub genre: String,
    pub titre: String,
    /// Similarité cosinus, de -1 à 1.
    pub score: f32,
}

/// Ce qui calcule les vecteurs : Ollama en vrai, un calcul déterministe dans les tests.
pub trait Plongeur {
    /// Nom du modèle, pris dans l'empreinte : changer de modèle recalcule tout.
    fn modele(&self) -> String;
    /// Un vecteur par texte, dans l'ordre.
    fn plonger(&self, textes: &[String]) -> Result<Vec<Vec<f32>>>;
}

/// Modèle d'embeddings d'Ollama, appelé par `curl` sur `/api/embed` (corps JSON sur l'entrée standard).
pub struct Ollama {
    pub url: String,
    pub modele: String,
    /// Délai maximum d'un appel, en secondes.
    pub delai: u64,
}

impl Ollama {
    /// Ollama de la configuration : `[semantique]`, sinon l'adresse d'Ollama du connecteur ou de Shelley.
    pub fn depuis(config: &Config) -> Self {
        let url = config.semantique.url.trim().trim_end_matches('/');
        Self {
            url: if url.is_empty() {
                crate::atelier::connecteurs::adresse_ollama(&config.atelier.llm)
            } else {
                url.to_string()
            },
            modele: config.semantique.modele.clone(),
            delai: config.atelier.llm.delai.max(120),
        }
    }
}

impl Plongeur for Ollama {
    /// Le modèle de `[semantique]`.
    fn modele(&self) -> String {
        self.modele.clone()
    }

    /// Un appel `/api/embed` pour tous les textes ; une erreur d'Ollama (modèle absent) est rendue avec le remède.
    fn plonger(&self, textes: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut enfant = Command::new("curl")
            .args([
                "-s",
                "-S",
                "--max-time",
                &self.delai.to_string(),
                "-X",
                "POST",
            ])
            .args([
                "-H",
                "Content-Type: application/json",
                "--data-binary",
                "@-",
            ])
            .arg(format!("{}/api/embed", self.url))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("lancement de curl")?;
        enfant.stdin.take().context("entrée de curl")?.write_all(
            json!({ "model": self.modele, "input": textes })
                .to_string()
                .as_bytes(),
        )?;
        let sortie = enfant.wait_with_output()?;
        if !sortie.status.success() {
            bail!(
                "Ollama injoignable à {} : {} (lance Ollama, ou règle [semantique] url dans wikik.toml)",
                self.url,
                String::from_utf8_lossy(&sortie.stderr).trim()
            );
        }
        let v: Value =
            serde_json::from_slice(&sortie.stdout).context("réponse d'Ollama illisible")?;
        if let Some(e) = v.get("error").and_then(Value::as_str) {
            bail!(
                "Ollama : {e} (installe le modèle : ollama pull {})",
                self.modele
            );
        }
        let vecteurs: Vec<Vec<f32>> =
            serde_json::from_value(v.get("embeddings").cloned().unwrap_or(Value::Null))
                .context("réponse d'Ollama sans `embeddings`")?;
        if vecteurs.len() != textes.len() {
            bail!(
                "Ollama a rendu {} vecteurs pour {} textes",
                vecteurs.len(),
                textes.len()
            );
        }
        Ok(vecteurs)
    }
}

/// Morceaux de l'index : chaque ligne de la table plein texte `recherche` (symboles documentés, documents).
pub fn morceaux(index: &Index) -> Result<Vec<Morceau>> {
    let mut st = index
        .conn
        .prepare("SELECT cle, genre, titre, texte FROM recherche ORDER BY cle")?;
    let v = st
        .query_map([], |r| {
            let titre: String = r.get(2)?;
            let texte: String = r.get(3)?;
            let complet = format!("{titre}\n{texte}");
            Ok(Morceau {
                cle: r.get(0)?,
                genre: r.get(1)?,
                titre,
                texte: complet.chars().take(LONGUEUR).collect(),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Fichier des vecteurs du projet.
pub fn chemin(racine: &Path) -> PathBuf {
    racine.join(DOSSIER).join("semantique.sqlite")
}

/// Ouvre (et crée au besoin) la base des vecteurs.
fn ouvrir(chemin: &Path) -> Result<Connection> {
    if let Some(d) = chemin.parent() {
        std::fs::create_dir_all(d)?;
    }
    let c = Connection::open(chemin)?;
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS vecteurs (cle TEXT PRIMARY KEY, empreinte TEXT NOT NULL, vecteur BLOB NOT NULL);",
    )?;
    Ok(c)
}

/// Vecteur en octets (f32 petit-boutiste) et retour.
fn en_octets(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}
/// Inverse de [`en_octets`].
fn depuis_octets(b: &[u8]) -> Vec<f32> {
    b.as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect()
}

/// Similarité cosinus ; 0 si l'un des vecteurs est nul ou si leurs tailles diffèrent.
fn cosinus(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let (mut p, mut na, mut nb) = (0f32, 0f32, 0f32);
    for (x, y) in a.iter().zip(b) {
        p += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        p / (na.sqrt() * nb.sqrt())
    }
}

/// Met les vecteurs à jour : calcule ceux des morceaux nouveaux ou modifiés, par lots, et retire ceux qui ont
/// disparu de l'index. Rend le nombre de vecteurs calculés.
pub fn mettre_a_jour(base: &Path, morceaux: &[Morceau], p: &dyn Plongeur) -> Result<usize> {
    let c = ouvrir(base)?;
    let modele = p.modele();
    let connues: HashMap<String, String> = c
        .prepare("SELECT cle, empreinte FROM vecteurs")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let empreinte =
        |m: &Morceau| crate::hash::empreinte_morceaux([modele.as_str(), m.texte.as_str()]);
    let a_faire: Vec<&Morceau> = morceaux
        .iter()
        .filter(|m| connues.get(&m.cle) != Some(&empreinte(m)))
        .collect();
    for lot in a_faire.chunks(LOT) {
        let textes: Vec<String> = lot.iter().map(|m| m.texte.clone()).collect();
        let vecteurs = p.plonger(&textes)?;
        let tx = c.unchecked_transaction()?;
        for (m, v) in lot.iter().zip(vecteurs) {
            tx.execute(
                "INSERT OR REPLACE INTO vecteurs (cle, empreinte, vecteur) VALUES (?1, ?2, ?3)",
                params![m.cle, empreinte(m), en_octets(&v)],
            )?;
        }
        tx.commit()?;
    }
    let presentes: std::collections::HashSet<&str> =
        morceaux.iter().map(|m| m.cle.as_str()).collect();
    for cle in connues.keys().filter(|k| !presentes.contains(k.as_str())) {
        c.execute("DELETE FROM vecteurs WHERE cle = ?1", [cle])?;
    }
    Ok(a_faire.len())
}

/// Les `n` morceaux les plus proches de la question, après mise à jour des vecteurs.
pub fn chercher(
    base: &Path,
    morceaux: &[Morceau],
    p: &dyn Plongeur,
    question: &str,
    n: usize,
) -> Result<Vec<Trouve>> {
    if question.trim().is_empty() {
        bail!("pose une question");
    }
    mettre_a_jour(base, morceaux, p)?;
    let q = p
        .plonger(&[question.trim().to_string()])?
        .pop()
        .context("pas de vecteur pour la question")?;
    let c = ouvrir(base)?;
    let vecteurs: HashMap<String, Vec<u8>> = c
        .prepare("SELECT cle, vecteur FROM vecteurs")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut v: Vec<Trouve> = morceaux
        .iter()
        .filter_map(|m| {
            let x = depuis_octets(vecteurs.get(&m.cle)?);
            Some(Trouve {
                cle: m.cle.clone(),
                genre: m.genre.clone(),
                titre: m.titre.clone(),
                score: cosinus(&q, &x),
            })
        })
        .collect();
    v.sort_by(|a, b| b.score.total_cmp(&a.score));
    v.truncate(n);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plongeur déterministe : un sac de mots haché sur 64 dimensions, et le nombre d'appels.
    struct Sac(std::cell::Cell<usize>);
    impl Plongeur for Sac {
        fn modele(&self) -> String {
            "sac".into()
        }
        fn plonger(&self, textes: &[String]) -> Result<Vec<Vec<f32>>> {
            self.0.set(self.0.get() + 1);
            Ok(textes
                .iter()
                .map(|t| {
                    let mut v = vec![0f32; 64];
                    for m in t.to_lowercase().split_whitespace() {
                        let h = crate::hash::empreinte(m.as_bytes());
                        v[usize::from_str_radix(&h[..4], 16).unwrap() % 64] += 1.0;
                    }
                    v
                })
                .collect())
        }
    }

    /// Morceau de test.
    fn m(cle: &str, texte: &str) -> Morceau {
        Morceau {
            cle: cle.into(),
            genre: "symbole".into(),
            titre: cle.into(),
            texte: texte.into(),
        }
    }

    #[test]
    fn plus_proche_et_mise_a_jour_partielle() {
        let d = tempfile::tempdir().unwrap();
        let base = d.path().join("s.sqlite");
        let p = Sac(Default::default());
        let ms = vec![
            m("a", "zoom du graphe molette"),
            m("b", "onglets aperçu fixés"),
            m("c", "remarque réception curseur"),
        ];
        let r = chercher(&base, &ms, &p, "molette zoom", 2).unwrap();
        assert_eq!(r[0].cle, "a");
        assert_eq!(r.len(), 2);
        // Rien n'a changé : seule la question est calculée.
        let appels = p.0.get();
        assert_eq!(mettre_a_jour(&base, &ms, &p).unwrap(), 0);
        assert_eq!(p.0.get(), appels);
        // Un morceau modifié, un disparu.
        let ms2 = vec![m("a", "zoom du graphe"), m("b", "onglets aperçu fixés")];
        assert_eq!(mettre_a_jour(&base, &ms2, &p).unwrap(), 1);
        let n: i64 = ouvrir(&base)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM vecteurs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn cosinus_borne() {
        assert!((cosinus(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
        assert_eq!(cosinus(&[0.0, 0.0], &[1.0, 0.0]), 0.0);
        assert_eq!(cosinus(&[1.0], &[1.0, 0.0]), 0.0);
        assert_eq!(depuis_octets(&en_octets(&[0.5, -2.0])), vec![0.5, -2.0]);
    }
}
