//! Édition des documents dans le navigateur (ADR-0016, EF-158) : lire le fichier tel quel, le remplacer en
//! entier (frontmatter et corps), ou changer son statut (« Valider » de la relecture) sans toucher au reste.
//! Seuls les documents de l'index, sous une zone de documents et en `.md`, s'écrivent ; chaque écriture
//! passe par le journal d'annulation.
//!
//! @req EF-158
//! @adr 16

use crate::index::Index;
use anyhow::{Context, Result, bail};
use rusqlite::OptionalExtension;
use std::path::PathBuf;

/// Chemin (relatif) d'un document de l'index, vérifié : dans une zone de documents, en `.md`.
fn chemin(index: &Index, id: &str) -> Result<(String, PathBuf)> {
    let rel: String = index
        .conn
        .query_row("SELECT chemin FROM documents WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?
        .with_context(|| format!("document inconnu : {id}"))?;
    let zones = index.config.zones_documents();
    if !rel.ends_with(".md")
        || rel.contains("..")
        || !zones.iter().any(|z| rel.starts_with(&format!("{z}/")))
    {
        bail!("{rel} n'est pas un document modifiable");
    }
    Ok((rel.clone(), index.racine.join(rel)))
}

/// Texte complet du document (frontmatter compris) et son chemin relatif.
pub fn source(index: &Index, id: &str) -> Result<(String, String)> {
    let (rel, abs) = chemin(index, id)?;
    Ok((rel, std::fs::read_to_string(abs)?))
}

/// Remplace le document par `texte`. Le frontmatter doit rester lisible et garder son `id`.
pub fn ecrire(index: &Index, id: &str, texte: &str) -> Result<String> {
    let (rel, abs) = chemin(index, id)?;
    if let Some(fm) = frontmatter(texte) {
        let v: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(fm).context("frontmatter illisible")?;
        let garde = v.get("id").and_then(|x| x.as_str());
        if garde.is_some_and(|g| g != id) {
            bail!("l'identifiant du frontmatter doit rester {id}");
        }
    }
    let mut t = texte.replace("\r\n", "\n");
    if !t.ends_with('\n') {
        t.push('\n');
    }
    crate::journal::ecrire(abs, t)?;
    Ok(rel)
}

/// Change le statut du document (`validé`, `proposé`…) dans son frontmatter, sans toucher au reste.
pub fn changer_statut(index: &Index, id: &str, statut: &str) -> Result<String> {
    let statut = statut.trim();
    if statut.is_empty() || statut.contains(['\n', ':']) {
        bail!("statut invalide : {statut}");
    }
    let (_, texte) = source(index, id)?;
    ecrire(index, id, &avec_champ(&texte, "statut", statut))
}

/// Dossier des dessins de l'atelier (EF-167) : `docs/atelier/dessins/`.
fn dossier_dessins(index: &Index) -> PathBuf {
    index
        .racine
        .join(index.config.atelier.dossier.trim_end_matches('/'))
        .join("dessins")
}

/// Nom de dessin sûr : lettres, chiffres, `-` et `_`, 80 caractères au plus ; refuse tout ce qui sortirait du dossier des dessins.
fn nom_dessin(nom: &str) -> Result<&str> {
    let ok = !nom.is_empty()
        && nom.len() <= 80
        && nom
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !ok {
        bail!("nom de dessin invalide : {nom}");
    }
    Ok(nom)
}

/// Enregistre un dessin vectoriel (vignette de storyboard, maquette d'interface) en SVG. Le SVG doit
/// rester un dessin : pas de script, d'attribut d'événement, de lien `javascript:` ni d'objet étranger.
/// @req EF-167
pub fn ecrire_dessin(index: &Index, nom: &str, svg: &str) -> Result<String> {
    let nom = nom_dessin(nom)?;
    let t = svg.trim();
    if !t.starts_with("<svg") || !t.ends_with("</svg>") || t.len() > 2_000_000 {
        bail!("un dessin est un élément <svg> complet (2 Mo au plus)");
    }
    let bas = t.to_lowercase();
    let interdit =
        regex::Regex::new(r#"<script|<foreignobject|<iframe|javascript:|\son[a-z]+\s*="#).unwrap();
    if interdit.is_match(&bas) {
        bail!("le dessin contient du code : refusé");
    }
    let f = dossier_dessins(index).join(format!("{nom}.svg"));
    crate::journal::ecrire(&f, format!("{t}\n"))?;
    Ok(f.strip_prefix(&index.racine)
        .unwrap_or(&f)
        .to_string_lossy()
        .into_owned())
}

/// Texte d'un dessin, `None` s'il n'existe pas.
pub fn lire_dessin(index: &Index, nom: &str) -> Result<Option<String>> {
    let f = dossier_dessins(index).join(format!("{}.svg", nom_dessin(nom)?));
    Ok(std::fs::read_to_string(f).ok())
}

/// Noms des dessins existants.
pub fn lister_dessins(index: &Index) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dossier_dessins(index))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_string_lossy()
                .strip_suffix(".svg")
                .map(str::to_string)
        })
        .collect();
    v.sort();
    v
}

/// Ticket de validation d'un test (EF-160) : `utilisateur` ou `modele` (validé), `echec`, ou `a_valider`
/// (rouvert) ; la date et l'auteur du verdict sont notés dans le frontmatter.
pub fn valider_test(index: &Index, id: &str, verdict: &str) -> Result<String> {
    if !matches!(verdict, "a_valider" | "utilisateur" | "modele" | "echec") {
        bail!("verdict inconnu : {verdict} (a_valider, utilisateur, modele, echec)");
    }
    let genre: Option<String> = index
        .conn
        .query_row("SELECT genre FROM documents WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?;
    if genre.as_deref() != Some("test") {
        bail!("{id} n'est pas un test");
    }
    let (_, mut texte) = source(index, id)?;
    texte = avec_champ(&texte, "validation", verdict);
    texte = avec_champ(&texte, "valide_le", &crate::relu::aujourdhui());
    ecrire(index, id, &texte)
}

/// Pose `cle: valeur` dans le frontmatter (ligne remplacée, ou ajoutée avant la fermeture).
pub fn avec_champ(texte: &str, cle: &str, valeur: &str) -> String {
    let Some(fm) = frontmatter(texte) else {
        return format!("---\n{cle}: {valeur}\n---\n\n{texte}");
    };
    let debut = texte.find(fm).unwrap_or(4);
    let mut lignes: Vec<String> = fm.lines().map(str::to_string).collect();
    let prefixe = format!("{cle}:");
    match lignes.iter_mut().find(|l| l.starts_with(&prefixe)) {
        Some(l) => *l = format!("{cle}: {valeur}"),
        None => lignes.push(format!("{cle}: {valeur}")),
    }
    format!(
        "{}{}\n{}",
        &texte[..debut],
        lignes.join("\n"),
        texte[debut + fm.len()..].trim_start_matches('\n')
    )
}

/// Contenu du frontmatter (entre les deux `---`), sans les séparateurs.
fn frontmatter(texte: &str) -> Option<&str> {
    let reste = texte.strip_prefix("---\n")?;
    let fin = reste.find("\n---")?;
    Some(&reste[..fin])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dessin_sans_code() {
        let d = tempfile::tempdir().unwrap();
        let i = Index::ouvrir(d.path()).unwrap();
        assert!(ecrire_dessin(&i, "SB-001-V1", "<svg><rect onclick=\"x()\"/></svg>").is_err());
        assert!(ecrire_dessin(&i, "../x", "<svg></svg>").is_err());
        ecrire_dessin(&i, "SB-001-V1", "<svg><rect width=\"10\"/></svg>").unwrap();
        assert_eq!(lister_dessins(&i), vec!["SB-001-V1".to_string()]);
        assert!(
            lire_dessin(&i, "SB-001-V1")
                .unwrap()
                .unwrap()
                .contains("rect")
        );
    }

    #[test]
    fn champ_remplace_ou_ajoute() {
        let t = "---\nid: B-1\nstatut: proposé\n---\n\n# B-1\n";
        assert_eq!(
            avec_champ(t, "statut", "validé"),
            "---\nid: B-1\nstatut: validé\n---\n\n# B-1\n"
        );
        assert_eq!(
            avec_champ("---\nid: B-1\n---\n# B\n", "statut", "validé"),
            "---\nid: B-1\nstatut: validé\n---\n# B\n"
        );
    }
}
