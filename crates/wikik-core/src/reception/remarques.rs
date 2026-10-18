//! Remarques de réception (SPEC-23) : une carte = une remarque = un fichier `REC-xxx` du dépôt, avec ses
//! images, rangée sous une page de la carte et reliée au code de cette page.
//!
//! Les images arrivent du presse-papiers en `data:` URL ; seules les vraies images PNG, JPEG, GIF et WebP
//! sont acceptées (types vérifiés sur les octets, pas sur l'annonce du navigateur), pour qu'aucun fichier
//! servi ne puisse exécuter de script.
//!
//! @req EF-123, EF-124, EF-125

use super::carte::{self, GENERAL};
use crate::index::Index;
use anyhow::{Context, Result, bail};
use base64::Engine;
use serde::Serialize;
use serde_json::{Map, Value, json};
use ts_rs::TS;

/// Genres d'une remarque.
pub const GENRES: &[&str] = &["bug", "ergonomie", "evolution", "question"];
/// Gravités, de la plus forte à la plus faible (ordre de traitement).
pub const GRAVITES: &[&str] = &["bloquant", "genant", "detail"];
/// Statuts : une remarque naît ouverte et finit traitée, reportée ou rejetée.
pub const STATUTS: &[&str] = &["ouverte", "traitee", "reportee", "rejetee"];
const TAILLE_MAX: usize = 10 * 1024 * 1024;

/// Remarque telle que l'écran et l'agent la lisent.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Remarque {
    pub id: String,
    pub titre: String,
    pub statut: String,
    pub page: String,
    pub nom_page: String,
    pub genre: String,
    pub gravite: String,
    pub texte: String,
    /// Chemins des images, relatifs à la racine du projet.
    pub images: Vec<String>,
    pub symboles: Vec<String>,
    pub exigences: Vec<String>,
    pub date: String,
    pub resolution: Option<String>,
    pub commit: Option<String>,
    pub chemin: String,
}

/// Type d'une image d'après ses premiers octets : `png`, `jpg`, `gif`, `webp`, sinon `None`.
pub fn type_image(octets: &[u8]) -> Option<&'static str> {
    match octets {
        [0x89, b'P', b'N', b'G', ..] => Some("png"),
        [0xFF, 0xD8, 0xFF, ..] => Some("jpg"),
        [b'G', b'I', b'F', b'8', ..] => Some("gif"),
        [
            b'R',
            b'I',
            b'F',
            b'F',
            _,
            _,
            _,
            _,
            b'W',
            b'E',
            b'B',
            b'P',
            ..,
        ] => Some("webp"),
        _ => None,
    }
}

/// Décode une `data:` URL d'image ; refuse ce qui n'est pas une image autorisée ou dépasse 10 Mo.
pub fn decoder(data_url: &str) -> Result<(Vec<u8>, &'static str)> {
    let b64 = data_url
        .split_once(";base64,")
        .map(|x| x.1)
        .context("image attendue en data: URL base64")?;
    let octets = base64::engine::general_purpose::STANDARD
        .decode(b64.trim())
        .context("base64 invalide")?;
    if octets.len() > TAILLE_MAX {
        bail!(
            "image trop lourde ({} Mo, 10 au plus)",
            octets.len() / 1024 / 1024
        );
    }
    let ext = type_image(&octets).context("format d'image refusé (PNG, JPEG, GIF ou WebP)")?;
    Ok((octets, ext))
}

fn dossier(index: &Index) -> String {
    index
        .config
        .reception
        .dossier
        .trim_end_matches('/')
        .to_string()
}

fn prochain_id(index: &Index) -> String {
    let rep = index.racine.join(dossier(index)).join("remarques");
    let max = std::fs::read_dir(rep)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_string_lossy()
                .strip_prefix("REC-")?
                .strip_suffix(".md")?
                .parse::<u32>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    format!("REC-{:03}", max + 1)
}

fn verifier(valeur: &str, permis: &[&str], quoi: &str) -> Result<()> {
    if !permis.contains(&valeur) {
        bail!("{quoi} inconnu(e) : {valeur} ({})", permis.join(", "));
    }
    Ok(())
}

/// Titre court : la première ligne du texte, 80 caractères au plus.
fn titre(texte: &str) -> String {
    let l = texte
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("Remarque")
        .trim();
    if l.chars().count() > 80 {
        format!("{}…", l.chars().take(80).collect::<String>().trim_end())
    } else {
        l.to_string()
    }
}

/// Enregistre une remarque sur une page de la carte (ou la page « Général ») ; renvoie son identifiant.
pub fn creer(
    index: &Index,
    page: &str,
    texte: &str,
    genre: &str,
    gravite: &str,
    images: &[String],
    date: &str,
) -> Result<String> {
    verifier(genre, GENRES, "genre")?;
    verifier(gravite, GRAVITES, "gravité")?;
    if texte.trim().is_empty() && images.is_empty() {
        bail!("la remarque est vide");
    }
    let pages = carte::lire(index);
    let p = pages.iter().find(|p| p.cle == page);
    if p.is_none() && page != GENERAL {
        bail!("page inconnue : {page} (scanner d'abord la carte)");
    }
    let decodees = images
        .iter()
        .map(|i| decoder(i))
        .collect::<Result<Vec<_>>>()?;
    let id = prochain_id(index);
    let racine = index.racine.join(dossier(index));
    std::fs::create_dir_all(racine.join("remarques"))?;
    std::fs::create_dir_all(racine.join("images"))?;
    let mut chemins = vec![];
    let mut corps = format!("# {id} : {}\n\n{}\n", titre(texte), texte.trim());
    for (n, (octets, ext)) in decodees.iter().enumerate() {
        let nom = format!("{id}-{}.{ext}", n + 1);
        std::fs::write(racine.join("images").join(&nom), octets)?;
        chemins.push(format!("{}/images/{nom}", dossier(index)));
        corps.push_str(&format!("\n![capture {}](../images/{nom})\n", n + 1));
    }
    let mut fm = Map::new();
    fm.insert("id".into(), json!(id));
    fm.insert("type".into(), json!("remarque"));
    fm.insert("titre".into(), json!(titre(texte)));
    fm.insert("statut".into(), json!("ouverte"));
    fm.insert("page".into(), json!(page));
    fm.insert(
        "nom_page".into(),
        json!(p.map_or("Général".to_string(), |p| p.nom.clone())),
    );
    fm.insert("genre".into(), json!(genre));
    fm.insert("gravite".into(), json!(gravite));
    fm.insert(
        "symboles".into(),
        json!(p.map(|p| p.symboles.clone()).unwrap_or_default()),
    );
    fm.insert(
        "exigences".into(),
        json!(p.map(|p| p.exigences.clone()).unwrap_or_default()),
    );
    fm.insert("images".into(), json!(chemins));
    fm.insert("date".into(), json!(date));
    std::fs::write(
        racine.join("remarques").join(format!("{id}.md")),
        crate::atelier::artefacts::formater(&fm, &corps),
    )?;
    Ok(id)
}

fn chaines(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Toutes les remarques (les plus récentes d'abord), lues dans l'index.
pub fn lister(index: &Index) -> Result<Vec<Remarque>> {
    let mut st = index.conn.prepare(
        "SELECT id, chemin, frontmatter FROM documents WHERE genre = 'remarque' ORDER BY id DESC",
    )?;
    let lignes: Vec<(String, String, String)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut v = vec![];
    for (id, chemin, fm) in lignes {
        let fm: Value = serde_json::from_str(&fm).unwrap_or_default();
        let t = |k: &str| {
            fm.get(k)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let corps = std::fs::read_to_string(index.racine.join(&chemin)).unwrap_or_default();
        let corps = crate::projet::separer_frontmatter(&corps).1;
        // Le texte : ce qui suit le titre, sans les images ni la section de traitement.
        let texte = corps
            .split("\n## Traitement")
            .next()
            .unwrap_or("")
            .lines()
            .skip_while(|l| !l.starts_with("# "))
            .skip(1)
            .filter(|l| !l.starts_with("![capture"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        v.push(Remarque {
            id,
            titre: t("titre"),
            statut: t("statut"),
            page: t("page"),
            nom_page: t("nom_page"),
            genre: t("genre"),
            gravite: t("gravite"),
            texte,
            images: chaines(fm.get("images")),
            symboles: chaines(fm.get("symboles")),
            exigences: chaines(fm.get("exigences")),
            date: t("date"),
            resolution: fm
                .get("resolution")
                .and_then(Value::as_str)
                .map(str::to_string),
            commit: fm.get("commit").and_then(Value::as_str).map(str::to_string),
            chemin,
        });
    }
    Ok(v)
}

/// Enregistre le traitement d'une remarque (statut, explication, commit éventuel) : frontmatter et
/// section « Traitement » ; le reste du fichier n'est pas touché.
pub fn traiter(
    index: &Index,
    id: &str,
    statut: &str,
    resolution: &str,
    commit: Option<&str>,
) -> Result<()> {
    verifier(statut, STATUTS, "statut")?;
    if statut != "ouverte" && resolution.trim().is_empty() {
        bail!("une explication est attendue pour passer {id} à « {statut} »");
    }
    let chemin: String = index
        .conn
        .query_row(
            "SELECT chemin FROM documents WHERE id = ?1 AND genre = 'remarque'",
            [id],
            |r| r.get(0),
        )
        .with_context(|| format!("remarque inconnue : {id}"))?;
    let (mut fm, corps) = crate::atelier::artefacts::lire(&index.racine, &chemin)?;
    fm.insert("statut".into(), json!(statut));
    if statut == "ouverte" {
        fm.remove("resolution");
        fm.remove("commit");
    } else {
        fm.insert("resolution".into(), json!(resolution.trim()));
        match commit.map(str::trim).filter(|c| !c.is_empty()) {
            Some(c) => fm.insert("commit".into(), json!(c)),
            None => fm.remove("commit"),
        };
    }
    let mut corps = corps
        .split("\n## Traitement")
        .next()
        .unwrap_or(&corps)
        .trim_end()
        .to_string();
    if statut != "ouverte" {
        corps.push_str(&format!(
            "\n\n## Traitement\n\n**{statut}** : {}\n",
            resolution.trim()
        ));
        if let Some(c) = commit.filter(|c| !c.trim().is_empty()) {
            corps.push_str(&format!("\nCommit : `{}`\n", c.trim()));
        }
    } else {
        corps.push('\n');
    }
    crate::atelier::artefacts::reecrire(&index.racine, &chemin, &fm, &corps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seules_les_vraies_images_passent() {
        let png = base64::engine::general_purpose::STANDARD.encode([0x89, b'P', b'N', b'G', 0, 0]);
        assert_eq!(
            decoder(&format!("data:image/png;base64,{png}")).unwrap().1,
            "png"
        );
        // Un SVG annoncé comme PNG est refusé : seuls les octets comptent.
        let svg = base64::engine::general_purpose::STANDARD.encode("<svg onload=alert(1)>");
        assert!(decoder(&format!("data:image/png;base64,{svg}")).is_err());
        assert!(decoder("pas une image").is_err());
        assert_eq!(
            titre("\n  Le bouton est mal placé\nsuite"),
            "Le bouton est mal placé"
        );
    }
}
