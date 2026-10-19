//! Phase 4 de la v4 : l'atelier est indexé, relié, contrôlé et exposé (SPEC-17, SPEC-21).

use std::path::Path;
use wikik_core::{Index, check, query};

fn copier(de: &Path, vers: &Path) {
    std::fs::create_dir_all(vers).unwrap();
    for e in std::fs::read_dir(de).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copier(&e.path(), &vers.join(e.file_name()));
        } else {
            std::fs::copy(e.path(), vers.join(e.file_name())).unwrap();
        }
    }
}

fn fixture() -> (tempfile::TempDir, Index) {
    let d = tempfile::tempdir().unwrap();
    copier(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/atelier-mini"),
        d.path(),
    );
    let mut i = Index::ouvrir(d.path()).unwrap();
    i.mettre_a_jour().unwrap();
    (d, i)
}

/// L'atelier est une zone de documents ; la chaîne exigence → besoin → storyboard → idée → unité est faite d'arêtes.
///
/// @req EF-105, EF-106
#[test]
fn chaine_de_tracabilite() {
    let (_d, i) = fixture();
    let aretes: Vec<(String, String)> = {
        let mut st = i
            .conn
            .prepare("SELECT source, cible FROM aretes WHERE genre = 'lien'")
            .unwrap();
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    for (s, c) in [
        ("EF-001", "B-001"),
        ("EF-001", "SB-001"),
        ("B-001", "SB-001"),
        ("SB-001", "IDEE-001"),
        ("IDEE-001", "US-001"),
    ] {
        assert!(
            aretes.contains(&(s.into(), c.into())),
            "{s} -> {c} absent de {aretes:?}"
        );
    }
    let a = query::atelier(&i).unwrap();
    assert!(
        a.texte.contains("IDEE-001 — Alerte brouillard [retenu]"),
        "{}",
        a.texte
    );
    let d = query::document_complet(&i, "IDEE-001").unwrap();
    assert!(
        d.texte.contains("technique : carte-mentale") && d.texte.contains("Prévenir"),
        "{}",
        d.texte
    );
}

/// Avec un atelier, une exigence sans besoin et un besoin sans source sont signalés.
///
/// @req EF-108
#[test]
fn controles_de_tracabilite() {
    let (_d, i) = fixture();
    let r = check::verifier(&i).unwrap();
    let codes: Vec<(&str, &str)> = r
        .diagnostics
        .iter()
        .map(|d| (d.code.as_str(), d.fichier.as_str()))
        .collect();
    assert!(
        codes.contains(&("exigence-sans-besoin", "docs/projet/exigences/EF-002.md")),
        "{codes:?}"
    );
    assert!(
        codes.contains(&("besoin-sans-source", "docs/projet/besoins/B-002.md")),
        "{codes:?}"
    );
    assert!(
        !codes
            .iter()
            .any(|(c, f)| c.contains("sans") && f.contains("EF-001")),
        "{codes:?}"
    );
}

/// Le rappel crée une clarification liée à son sujet, et refuse un sujet inconnu.
///
/// @req EF-110
#[test]
fn rappel_cree_une_clarification() {
    let (d, mut i) = fixture();
    let (id, url) = wikik_core::atelier::rappel(&i, "EF-001", "Pourquoi 200 m ?").unwrap();
    assert_eq!(
        (id.as_str(), url.as_str()),
        ("CLAR-001", "http://127.0.0.1:8484/clarification/CLAR-001")
    );
    assert!(
        d.path()
            .join("docs/atelier/clarifications/CLAR-001.md")
            .exists()
    );
    assert!(wikik_core::atelier::rappel(&i, "EF-999", "?").is_err());
    i.mettre_a_jour().unwrap();
    let e = query::exigence(&i, "EF-001").unwrap();
    assert!(
        e.texte.contains("CLAR-001"),
        "la clarification apparaît dans les documents liés : {}",
        e.texte
    );
}
