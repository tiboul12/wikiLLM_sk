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
// Une version modifiée passe à `remplacee` (SPEC-23 R8) : statut posé par `modifier`, jamais par `traiter`.
/// Taille maximale d'une capture décodée (10 Mo), comme côté écran.
const TAILLE_MAX: usize = 10 * 1024 * 1024;

/// Remarque telle que l'écran et l'agent la lisent, avec sa place dans la chaîne des versions.
/// @relu 2026-10-22 038851cd
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
    /// Validation de l'utilisateur après traitement (EF-165) : `validee`, ou absente.
    pub validation: Option<String>,
    /// Date de la validation, ou de la dernière demande de reprise.
    pub valide_le: Option<String>,
    pub chemin: String,
    /// 1 pour la première saisie, puis 2, 3… à chaque modification (SPEC-23 R8).
    pub version: u32,
    pub remplace: Option<String>,
    pub remplace_par: Option<String>,
    /// Toutes les versions de la chaîne, de la plus ancienne à la plus récente.
    pub historique: Vec<String>,
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

/// Dossier du cahier de réception (`[reception] dossier`), sans `/` final.
fn dossier(index: &Index) -> String {
    index
        .config
        .reception
        .dossier
        .trim_end_matches('/')
        .to_string()
}

/// Prochain numéro : au-delà des fichiers présents et du dernier numéro noté dans la carte (un numéro
/// n'est jamais réutilisé, même après suppression).
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
                .split('.')
                .next()?
                .parse::<u32>()
                .ok()
        })
        .max()
        .unwrap_or(0)
        .max(carte::dernier_numero(index));
    format!("REC-{:03}", max + 1)
}

/// Refuse une valeur hors de la liste permise (genre, gravité, statut) avec un message qui liste les valeurs admises.
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

/// Contenu saisi d'une remarque, à la création comme pour une nouvelle version.
pub struct Saisie<'a> {
    pub page: &'a str,
    pub texte: &'a str,
    pub genre: &'a str,
    pub gravite: &'a str,
    /// Nouvelles images, en `data:` URL.
    pub images: &'a [String],
    /// Images déjà enregistrées à garder (chemins relatifs à la racine), pour une nouvelle version.
    pub garder: &'a [String],
    pub date: &'a str,
}

/// Chemin relatif du fichier d'une remarque.
fn chemin_remarque(index: &Index, id: &str) -> String {
    format!("{}/remarques/{id}.md", dossier(index))
}

/// Enregistre une remarque sur une page de la carte (ou la page « Général ») ; renvoie son identifiant.
/// @relu 2026-10-18 fc531a17
pub fn creer(index: &Index, s: &Saisie) -> Result<String> {
    let id = prochain_id(index);
    ecrire_version(index, &id, s, 1, None, "ouverte")?;
    if let Some(n) = id.strip_prefix("REC-").and_then(|n| n.parse().ok()) {
        carte::noter_numero(index, n)?;
    }
    Ok(id)
}

/// Écrit une version de remarque : frontmatter, texte, images gardées puis nouvelles.
/// @relu 2026-10-22 d05af9ad
fn ecrire_version(
    index: &Index,
    id: &str,
    s: &Saisie,
    version: u32,
    remplace: Option<&str>,
    statut: &str,
) -> Result<()> {
    verifier(s.genre, GENRES, "genre")?;
    verifier(s.gravite, GRAVITES, "gravité")?;
    if s.texte.trim().is_empty() && s.images.is_empty() && s.garder.is_empty() {
        bail!("la remarque est vide");
    }
    let pages = carte::lire(index);
    let p = pages.iter().find(|p| p.cle == s.page);
    if p.is_none() && s.page != GENERAL {
        bail!("page inconnue : {} (scanner d'abord la carte)", s.page);
    }
    let decodees = s
        .images
        .iter()
        .map(|i| decoder(i))
        .collect::<Result<Vec<_>>>()?;
    let racine = index.racine.join(dossier(index));
    std::fs::create_dir_all(racine.join("remarques"))?;
    std::fs::create_dir_all(racine.join("images"))?;
    let mut chemins: Vec<String> = s.garder.to_vec();
    for (n, (octets, ext)) in decodees.iter().enumerate() {
        let nom = format!("{id}-{}.{ext}", n + 1);
        crate::journal::ecrire(racine.join("images").join(&nom), octets)?;
        chemins.push(format!("{}/images/{nom}", dossier(index)));
    }
    let mut corps = format!("# {id} : {}\n\n{}\n", titre(s.texte), s.texte.trim());
    for (n, c) in chemins.iter().enumerate() {
        let nom = c.rsplit('/').next().unwrap_or(c);
        corps.push_str(&format!("\n![capture {}](../images/{nom})\n", n + 1));
    }
    let mut fm = Map::new();
    fm.insert("id".into(), json!(id));
    fm.insert("type".into(), json!("remarque"));
    fm.insert("titre".into(), json!(titre(s.texte)));
    fm.insert("statut".into(), json!(statut));
    fm.insert("page".into(), json!(s.page));
    fm.insert(
        "nom_page".into(),
        json!(p.map_or("Général".to_string(), |p| p.nom.clone())),
    );
    fm.insert("genre".into(), json!(s.genre));
    fm.insert("gravite".into(), json!(s.gravite));
    fm.insert(
        "symboles".into(),
        json!(p.map(|p| p.symboles.clone()).unwrap_or_default()),
    );
    fm.insert(
        "exigences".into(),
        json!(p.map(|p| p.exigences.clone()).unwrap_or_default()),
    );
    fm.insert("images".into(), json!(chemins));
    fm.insert("date".into(), json!(s.date));
    if version > 1 {
        fm.insert("version".into(), json!(version));
    }
    if let Some(r) = remplace {
        fm.insert("remplace".into(), json!(r));
    }
    crate::journal::ecrire(
        index.racine.join(chemin_remarque(index, id)),
        crate::atelier::artefacts::formater(&fm, &corps),
    )?;
    Ok(())
}

/// (base, version) d'un identifiant : `REC-001` → (`REC-001`, 1), `REC-001.3` → (`REC-001`, 3).
fn decomposer(id: &str) -> (&str, u32) {
    match id.rsplit_once('.') {
        Some((base, v)) if v.chars().all(|c| c.is_ascii_digit()) && !v.is_empty() => {
            (base, v.parse().unwrap_or(1))
        }
        _ => (id, 1),
    }
}

/// Frontmatter et corps d'une remarque ; erreur claire si elle n'existe pas.
fn lire_fm(index: &Index, id: &str) -> Result<(Map<String, Value>, String)> {
    let chemin = chemin_remarque(index, id);
    if !index.racine.join(&chemin).exists() {
        bail!("remarque inconnue : {id}");
    }
    crate::atelier::artefacts::lire(&index.racine, &chemin)
}

/// Modifie une remarque en écrivant sa version suivante (`REC-001` → `REC-001.2`, SPEC-23 R8) : la
/// nouvelle reprend le statut et le traitement, l'ancienne passe à `remplacee` ; renvoie le nouvel
/// identifiant.
pub fn modifier(index: &Index, id: &str, s: &Saisie) -> Result<String> {
    let (mut ancien, corps_ancien) = lire_fm(index, id)?;
    let statut = ancien
        .get("statut")
        .and_then(Value::as_str)
        .unwrap_or("ouverte")
        .to_string();
    if statut == "remplacee" {
        let par = ancien
            .get("remplace_par")
            .and_then(Value::as_str)
            .unwrap_or("?");
        bail!("{id} a déjà été remplacée par {par} : modifier la dernière version");
    }
    let images_anciennes = chaines(ancien.get("images"));
    if let Some(i) = s.garder.iter().find(|g| !images_anciennes.contains(g)) {
        bail!("image inconnue de {id} : {i}");
    }
    let (base, _) = decomposer(id);
    let suivante = std::fs::read_dir(index.racine.join(dossier(index)).join("remarques"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let n = e
                .file_name()
                .to_string_lossy()
                .strip_suffix(".md")?
                .to_string();
            let (b, v) = decomposer(&n);
            (b == base).then_some(v)
        })
        .max()
        .unwrap_or(1)
        + 1;
    let nouveau = format!("{base}.{suivante}");
    ecrire_version(index, &nouveau, s, suivante, Some(id), "ouverte")?;
    if statut != "ouverte" {
        let resolution = ancien
            .get("resolution")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let commit = ancien
            .get("commit")
            .and_then(Value::as_str)
            .map(str::to_string);
        traiter(index, &nouveau, &statut, &resolution, commit.as_deref())?;
    }
    ancien.insert("statut".into(), json!("remplacee"));
    ancien.insert("remplace_par".into(), json!(nouveau));
    crate::atelier::artefacts::reecrire(
        &index.racine,
        &chemin_remarque(index, id),
        &ancien,
        &corps_ancien,
    )?;
    Ok(nouveau)
}

/// Supprime une remarque et toute sa chaîne de versions (SPEC-23 R9), avec les images qu'aucune autre
/// remarque ne cite ; renvoie les identifiants supprimés, du plus ancien au plus récent.
/// @relu 2026-10-22 37af892e
pub fn supprimer(index: &Index, id: &str) -> Result<Vec<String>> {
    let lien = |id: &str, cle: &str| -> Option<String> {
        lire_fm(index, id)
            .ok()?
            .0
            .get(cle)?
            .as_str()
            .map(str::to_string)
    };
    lire_fm(index, id)?;
    let mut chaine = vec![id.to_string()];
    let mut c = id.to_string();
    while let Some(avant) = lien(&c, "remplace").filter(|a| !chaine.contains(a)) {
        chaine.insert(0, avant.clone());
        c = avant;
    }
    let mut c = id.to_string();
    while let Some(apres) = lien(&c, "remplace_par").filter(|a| !chaine.contains(a)) {
        chaine.push(apres.clone());
        c = apres;
    }
    let mut images = vec![];
    for i in &chaine {
        if let Ok((fm, _)) = lire_fm(index, i) {
            images.extend(chaines(fm.get("images")));
        }
        let _ = crate::journal::supprimer(index.racine.join(chemin_remarque(index, i)));
    }
    // Une image encore citée par une autre remarque reste.
    let restantes: Vec<String> =
        std::fs::read_dir(index.racine.join(dossier(index)).join("remarques"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| std::fs::read_to_string(e.path()).ok())
            .collect();
    for img in images {
        let nom = img.rsplit('/').next().unwrap_or(&img).to_string();
        if !restantes.iter().any(|t| t.contains(&nom))
            && img.starts_with(&format!("{}/images/", dossier(index)))
        {
            let _ = crate::journal::supprimer(index.racine.join(&img));
        }
    }
    Ok(chaine)
}

/// Liste de chaînes d'une valeur JSON ; le reste est ignoré.
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

/// Validation par l'utilisateur d'une remarque traitée (EF-165, REC-079) : `validee` la confirme ;
/// `a_reprendre` la rouvre pour l'agent, avec le commentaire de l'utilisateur ajouté à son texte (l'outil
/// `reception` le lui montre).
pub fn valider(index: &Index, id: &str, verdict: &str, commentaire: &str) -> Result<()> {
    let chemin = chemin_remarque(index, id);
    let (mut fm, corps) = lire_fm(index, id)?;
    let statut = fm
        .get("statut")
        .and_then(Value::as_str)
        .unwrap_or("ouverte")
        .to_string();
    match verdict {
        "validee" => {
            if statut == "ouverte" || statut == "remplacee" {
                bail!("{id} n'est pas traitée : rien à valider");
            }
            fm.insert("validation".into(), json!("validee"));
            fm.insert("valide_le".into(), json!(crate::relu::aujourdhui()));
            crate::atelier::artefacts::reecrire(&index.racine, &chemin, &fm, &corps)
        }
        "a_reprendre" => {
            if commentaire.trim().is_empty() {
                bail!("dis ce qui reste à reprendre");
            }
            let mut texte = corps
                .split(
                    "
## Traitement",
                )
                .next()
                .unwrap_or(&corps)
                .trim_end()
                .to_string();
            let precedent = fm
                .get("resolution")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            texte.push_str(&format!(
                "

**À reprendre** ({}) : {}{}
",
                crate::relu::aujourdhui(),
                commentaire.trim(),
                if precedent.is_empty() {
                    String::new()
                } else {
                    format!(" (traitement précédent : {precedent})")
                }
            ));
            fm.insert("statut".into(), json!("ouverte"));
            fm.insert("valide_le".into(), json!(crate::relu::aujourdhui()));
            for k in ["validation", "resolution", "commit"] {
                fm.remove(k);
            }
            crate::atelier::artefacts::reecrire(&index.racine, &chemin, &fm, &texte)
        }
        autre => bail!("verdict inconnu : {autre} (validee ou a_reprendre)"),
    }
}

/// Toutes les remarques (les plus récentes d'abord), lues dans l'index, versions remplacées comprises,
/// chacune avec l'historique de sa chaîne.
/// @relu 2026-10-22 ff30d620
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
            validation: fm
                .get("validation")
                .and_then(Value::as_str)
                .map(str::to_string),
            valide_le: fm
                .get("valide_le")
                .and_then(Value::as_str)
                .map(str::to_string),
            chemin,
            version: fm
                .get("version")
                .and_then(Value::as_u64)
                .map_or(1, |v| v as u32),
            remplace: fm
                .get("remplace")
                .and_then(Value::as_str)
                .map(str::to_string),
            remplace_par: fm
                .get("remplace_par")
                .and_then(Value::as_str)
                .map(str::to_string),
            historique: vec![],
        });
    }
    // Historique : la chaîne remplace → remplace_par de chaque remarque.
    let suivant: std::collections::BTreeMap<String, String> = v
        .iter()
        .filter_map(|r| Some((r.id.clone(), r.remplace_par.clone()?)))
        .collect();
    let precedent: std::collections::BTreeMap<String, String> = v
        .iter()
        .filter_map(|r| Some((r.id.clone(), r.remplace.clone()?)))
        .collect();
    for r in &mut v {
        let mut debut = r.id.clone();
        let mut garde = 0;
        while let Some(p) = precedent.get(&debut) {
            debut = p.clone();
            garde += 1;
            if garde > 100 {
                break;
            }
        }
        let mut chaine = vec![debut.clone()];
        while let Some(n) = suivant.get(chaine.last().unwrap()) {
            if chaine.contains(n) || chaine.len() > 100 {
                break;
            }
            chaine.push(n.clone());
        }
        r.historique = if chaine.len() > 1 { chaine } else { vec![] };
    }
    Ok(v)
}

/// Enregistre le traitement d'une remarque (statut, explication, commit éventuel) : frontmatter et
/// section « Traitement » ; le reste du fichier n'est pas touché. Une version remplacée ne se traite plus.
/// @relu 2026-10-22 ad88866a
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
    let chemin = chemin_remarque(index, id);
    let (mut fm, corps) = lire_fm(index, id)?;
    if fm.get("statut").and_then(Value::as_str) == Some("remplacee") {
        let par = fm
            .get("remplace_par")
            .and_then(Value::as_str)
            .unwrap_or("?");
        bail!("{id} a été remplacée par {par} : traiter la dernière version");
    }
    fm.insert("statut".into(), json!(statut));
    // Un nouveau traitement attend une nouvelle validation de l'utilisateur.
    fm.remove("validation");
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
