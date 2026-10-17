//! Mesure de vitesse (ENF-003) sur un projet synthétique de 50 000 lignes.
//! Ignorée par défaut : `cargo test --release -p wikik-core --test vitesse -- --ignored --nocapture`.

use std::time::Instant;
use wikik_core::Index;

/// Premier index < 2 s, mise à jour d'un fichier < 200 ms, en build release.
///
/// @req ENF-003
#[test]
#[ignore]
fn cinquante_mille_lignes() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("src")).unwrap();
    let mut total = 0;
    for f in 0..100 {
        let mut code = format!("//! Module {f}.\n\n");
        for i in 0..50 {
            code.push_str(&format!(
                "/// Fonction {i} du module {f}.\npub fn f{i}(x: u64) -> u64 {{\n    let y = x.wrapping_mul({i});\n    if y > 10 {{\n        f{j}(y / 2)\n    }} else {{\n        y\n    }}\n}}\n\n",
                j = (i + 1) % 50
            ));
        }
        total += code.lines().count();
        std::fs::write(d.path().join(format!("src/m{f}.rs")), code).unwrap();
    }
    let t = Instant::now();
    let mut i = Index::ouvrir(d.path()).unwrap();
    i.mettre_a_jour().unwrap();
    let premier = t.elapsed();
    std::fs::write(
        d.path().join("src/m0.rs"),
        "//! Module 0 modifié.\n\n/// Seule fonction.\npub fn g() {}\n",
    )
    .unwrap();
    let t = Instant::now();
    let r = i.mettre_a_jour().unwrap();
    let maj = t.elapsed();
    eprintln!(
        "{total} lignes : premier index {premier:?}, mise à jour d'un fichier {maj:?} ({} analysé)",
        r.analyses
    );
    assert_eq!(r.analyses, 1);
    if !cfg!(debug_assertions) {
        assert!(premier.as_millis() < 2000 && maj.as_millis() < 200);
    }
}
