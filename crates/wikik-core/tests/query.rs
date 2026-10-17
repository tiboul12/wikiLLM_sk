//! Critères d'acceptation de SPEC-07 sur la fixture `rust-mini`.

use std::path::Path;
use wikik_core::{Index, query};

fn index_fixture() -> (tempfile::TempDir, Index) {
    let dest = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/rust-mini");
    copier(&source, dest.path());
    let mut i = Index::ouvrir(dest.path()).unwrap();
    i.mettre_a_jour().unwrap();
    (dest, i)
}

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

#[test]
fn entete_avec_cout() {
    let (_d, i) = index_fixture();
    for s in [
        query::carte(&i, false).unwrap(),
        query::fichier(&i, "geo.rs").unwrap(),
    ] {
        let premiere = s.texte.lines().next().unwrap();
        assert!(
            premiere.starts_with("# ") && premiere.ends_with(" tokens"),
            "{premiere}"
        );
    }
}

#[test]
fn impact_par_distance() {
    let (_d, i) = index_fixture();
    let s = query::impact(&i, "src/geo.rs::Point::carre", 3).unwrap();
    assert!(
        s.texte.contains("1  src/geo.rs::Point::norme"),
        "{}",
        s.texte
    );
    assert!(
        s.texte.contains("2  src/lib.rs::entree (probable)"),
        "{}",
        s.texte
    );
    assert!(
        s.texte.contains("ADR-0001") && s.texte.contains("EF-001"),
        "{}",
        s.texte
    );
    assert!(s.texte.contains("piège (norme)"), "{}", s.texte);
}

#[test]
fn identifiants_partiels() {
    let (_d, i) = index_fixture();
    let s = query::symbole(&i, "norme", false).unwrap();
    assert!(!s.ambigu && s.texte.contains("src/geo.rs::Point::norme"));
    assert!(!s.texte.contains("```"), "pas de code sans --code");
    let amb = query::symbole(&i, "exe", false).unwrap();
    assert!(amb.ambigu);
    assert!(amb.texte.contains("exe@cfg(windows)") && amb.texte.contains("exe@cfg(not(windows))"));
    let code = query::symbole(&i, "aire", true).unwrap();
    assert!(code.texte.contains("cote * cote"));
}

#[test]
fn exigence_adr_cherche_ref() {
    let (_d, i) = index_fixture();
    let e = query::exigence(&i, "ef-001").unwrap();
    assert!(
        e.texte.contains("implémentée par src/lib.rs::entree"),
        "{}",
        e.texte
    );
    assert!(e.texte.contains("B-001 Calculer une aire"), "{}", e.texte);
    let a = query::adr(&i, "1").unwrap();
    assert!(
        a.texte.contains("cité par src/geo.rs::Point::norme"),
        "{}",
        a.texte
    );
    let c = query::cherche(&i, "aire").unwrap();
    assert!(c.texte.contains("symbole src/geo.rs::aire"), "{}", c.texte);
    let r = query::reference(&i, "@wiki:rust-mini/src/geo.rs::Point::norme").unwrap();
    assert!(
        r.texte.contains("# symbole src/geo.rs::Point::norme"),
        "{}",
        r.texte
    );
    let r = query::reference(&i, "@wiki:exigences/EF-001").unwrap();
    assert!(r.texte.contains("# document EF-001"), "{}", r.texte);
}

#[test]
fn ancres_identiques_au_wiki() {
    assert_eq!(query::ancre("EF-001 : Aire doublée"), "ef-001-aire-doublee");
    assert_eq!(
        query::ancre("1. Ce que la v1 doit livrer"),
        "1-ce-que-la-v1-doit-livrer"
    );
    let (_d, i) = index_fixture();
    let r = query::reference(&i, "@wiki:rust-mini/EF-001#ef-001-aire-doublee").unwrap();
    assert!(r.texte.contains("Énoncé"), "{}", r.texte);
}
