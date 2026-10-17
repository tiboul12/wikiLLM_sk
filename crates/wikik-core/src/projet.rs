//! Lecture des documents de la zone Projet : frontmatter YAML, titre, liens `[[...]]` (SPEC-03).
//!
//! @req EF-005

use crate::hash::empreinte;
use serde_json::Value;

/// Champs de frontmatter dont les valeurs sont des identifiants de documents (arêtes `lien`).
pub const CHAMPS_LIENS: &[&str] = &[
    "besoins",
    "exigences",
    "decisions",
    "sources",
    "remplace",
    "remplace_par",
];

/// Document Markdown de la zone Projet.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub id: String,
    /// Chemin relatif à la racine du projet.
    pub chemin: String,
    pub genre: String,
    pub titre: String,
    pub statut: Option<String>,
    pub version: Option<String>,
    pub priorite: Option<String>,
    /// Frontmatter en JSON canonique (clés triées) ; `{}` sans frontmatter.
    pub frontmatter: Value,
    pub frontmatter_invalide: Option<String>,
    pub corps: String,
    pub hash_contenu: String,
    pub liens: Vec<LienBrut>,
}

/// Lien `[[cible]]` du corps, ou valeur d'un champ de liens du frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LienBrut {
    pub cible: String,
    pub ligne: usize,
}

/// `3`, `003`, `ADR-3`, `adr-0003` → `ADR-0003` ; `None` si ce n'est pas un numéro d'ADR.
pub fn normaliser_adr(v: &str) -> Option<String> {
    let t = v.trim();
    let num = t
        .strip_prefix("ADR-")
        .or_else(|| t.strip_prefix("adr-"))
        .unwrap_or(t);
    let n: u32 = num.parse().ok()?;
    Some(format!("ADR-{n:04}"))
}

/// Analyse un document. `relatif_projet` est le chemin relatif au dossier de la zone Projet,
/// qui sert d'identifiant quand le frontmatter n'en donne pas.
pub fn lire(chemin: &str, relatif_projet: &str, contenu: &str) -> Document {
    let (yaml, corps, decalage) = separer_frontmatter(contenu);
    let (frontmatter, frontmatter_invalide) = match yaml {
        None => (Value::Object(Default::default()), None),
        Some(y) => match serde_yaml_ng::from_str::<Value>(y) {
            Ok(Value::Object(m)) => (Value::Object(m), None),
            Ok(Value::Null) => (Value::Object(Default::default()), None),
            Ok(_) => (
                Value::Object(Default::default()),
                Some("le frontmatter n'est pas un dictionnaire".into()),
            ),
            Err(e) => (Value::Object(Default::default()), Some(e.to_string())),
        },
    };
    let champ = |k: &str| frontmatter.get(k).and_then(valeur_texte);
    let genre = champ("type").unwrap_or_else(|| "page".into());
    let sans_ext = relatif_projet
        .strip_suffix(".md")
        .unwrap_or(relatif_projet)
        .to_string();
    let id = match champ("id") {
        Some(i) if genre == "adr" || i.to_ascii_uppercase().starts_with("ADR") => {
            normaliser_adr(&i).unwrap_or(i)
        }
        Some(i) => i,
        None => sans_ext.clone(),
    };
    let titre = champ("titre")
        .or_else(|| {
            corps
                .lines()
                .find_map(|l| l.strip_prefix("# ").map(|t| t.trim().to_string()))
        })
        .unwrap_or_else(|| sans_ext.rsplit('/').next().unwrap_or(&sans_ext).to_string());

    let mut liens = liens_du_corps(corps, decalage);
    for &c in CHAMPS_LIENS {
        let Some(v) = frontmatter.get(c) else {
            continue;
        };
        let valeurs: Vec<String> = match v {
            Value::Array(a) => a.iter().filter_map(valeur_texte).collect(),
            autre => valeur_texte(autre).into_iter().collect(),
        };
        for val in valeurs {
            // Un chemin de fichier (hors zone Projet) est informatif, jamais vérifié : son
            // existence dépendrait de la machine (ENF-002).
            if val.contains('/') || val.ends_with(".md") {
                continue;
            }
            liens.push(LienBrut {
                cible: val,
                ligne: 1,
            });
        }
    }

    Document {
        id,
        chemin: chemin.to_string(),
        genre,
        titre,
        statut: champ("statut"),
        version: champ("version"),
        priorite: champ("priorite"),
        frontmatter,
        frontmatter_invalide,
        corps: corps.to_string(),
        hash_contenu: empreinte(contenu.as_bytes()),
        liens,
    }
}

fn valeur_texte(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Renvoie (yaml, corps, nombre de lignes avant le corps).
fn separer_frontmatter(contenu: &str) -> (Option<&str>, &str, usize) {
    let Some(reste) = contenu
        .strip_prefix("---\n")
        .or_else(|| contenu.strip_prefix("---\r\n"))
    else {
        return (None, contenu, 0);
    };
    let mut pos = 0;
    for (i, ligne) in reste.split_inclusive('\n').enumerate() {
        if ligne.trim_end() == "---" {
            let yaml = &reste[..pos];
            let corps = &reste[pos + ligne.len()..];
            return (Some(yaml), corps, i + 2);
        }
        pos += ligne.len();
    }
    (None, contenu, 0)
}

/// Liens `[[cible]]`, `[[cible|libellé]]`, `[[cible#ancre]]`, hors blocs et spans de code.
fn liens_du_corps(corps: &str, decalage: usize) -> Vec<LienBrut> {
    let mut v = vec![];
    let mut dans_bloc: Option<&str> = None;
    for (i, ligne) in corps.lines().enumerate() {
        let t = ligne.trim_start();
        let barriere = if t.starts_with("```") {
            Some("```")
        } else if t.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };
        if let Some(b) = barriere {
            match dans_bloc {
                Some(ouvert) if ouvert == b => dans_bloc = None,
                None => dans_bloc = Some(b),
                _ => {}
            }
            continue;
        }
        if dans_bloc.is_some() {
            continue;
        }
        let hors_code: String = ligne.split('`').step_by(2).collect::<Vec<_>>().join(" ");
        let mut reste = hors_code.as_str();
        while let Some(d) = reste.find("[[") {
            let apres = &reste[d + 2..];
            let Some(f) = apres.find("]]") else { break };
            let brut = &apres[..f];
            let cible = brut
                .split('|')
                .next()
                .unwrap_or("")
                .split('#')
                .next()
                .unwrap_or("")
                .trim();
            if !cible.is_empty() {
                v.push(LienBrut {
                    cible: cible.to_string(),
                    ligne: decalage + i + 1,
                });
            }
            reste = &apres[f + 2..];
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_liens_et_titre() {
        let d = lire(
            "docs/projet/exigences/EF-001.md",
            "exigences/EF-001.md",
            "---\nid: EF-001\ntype: exigence\ntitre: \"Aire : doublée\"\nbesoins: [B-001]\nsources: [wiki/x.md]\n---\n\n# Ignoré\nVoir [[adr/0001|l'ADR]] et `[[pas-un-lien]]`.\n```\n[[non]]\n```\n",
        );
        assert_eq!(d.id, "EF-001");
        assert_eq!(d.titre, "Aire : doublée");
        let cibles: Vec<_> = d
            .liens
            .iter()
            .map(|l| (l.cible.as_str(), l.ligne))
            .collect();
        assert_eq!(cibles, vec![("adr/0001", 10), ("B-001", 1)]);
    }

    #[test]
    fn sans_frontmatter_et_adr() {
        let d = lire("docs/projet/notes.md", "notes.md", "# Notes\ntexte");
        assert_eq!(
            (d.id.as_str(), d.genre.as_str(), d.titre.as_str()),
            ("notes", "page", "Notes")
        );
        let a = lire("x", "adr/1.md", "---\nid: 7\ntype: adr\n---\n");
        assert_eq!(a.id, "ADR-0007");
        let invalide = lire("x", "y.md", "---\ntitre: a: b: c\n---\n");
        assert!(invalide.frontmatter_invalide.is_some());
    }
}
