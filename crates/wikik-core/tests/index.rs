//! Tests d'intégration de l'index sur la fixture `tests/fixtures/rust-mini` (critères de SPEC-02 et SPEC-03).
//!
//! L'instantané `tests/fixtures/rust-mini.dump.json` se régénère avec `WIKIK_BENIR=1 cargo test`.

use serde_json::Value;
use std::path::{Path, PathBuf};
use wikik_core::Index;

/// Racine du dépôt wikiK, pour indexer ses fixtures et le dépôt lui-même.
fn racine_depot() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Copie récursive de la fixture dans un dossier temporaire : l'index n'est jamais écrit dans le dépôt.
fn copie_fixture(nom: &str) -> tempfile::TempDir {
    let dest = tempfile::tempdir().unwrap();
    copier(
        &racine_depot().join("tests/fixtures").join(nom),
        dest.path(),
    );
    dest
}

/// Copie récursive d'un dossier de fixtures dans un dossier temporaire, pour que le test écrive sans toucher au dépôt.
fn copier(de: &Path, vers: &Path) {
    std::fs::create_dir_all(vers).unwrap();
    for e in std::fs::read_dir(de).unwrap() {
        let e = e.unwrap();
        let cible = vers.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copier(&e.path(), &cible);
        } else {
            std::fs::copy(e.path(), cible).unwrap();
        }
    }
}

/// Indexe un dossier : rapport de mise à jour et contenu de l'index en JSON.
fn indexer(racine: &Path) -> (wikik_core::Rapport, Value) {
    let mut i = Index::ouvrir(racine).unwrap();
    let r = i.mettre_a_jour().unwrap();
    (r, i.dump_json().unwrap())
}

/// Symbole du dump par identifiant ; panique s'il manque.
fn symbole<'a>(dump: &'a Value, id: &str) -> &'a Value {
    dump["symboles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == id)
        .unwrap_or_else(|| panic!("symbole absent : {id}"))
}

/// Arêtes d'un genre du dump : (source, cible, confiance).
fn aretes(dump: &Value, genre: &str) -> Vec<(String, String, String)> {
    dump["aretes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["genre"] == genre)
        .map(|a| {
            (
                a["source"].as_str().unwrap().into(),
                a["cible"].as_str().unwrap().into(),
                a["confiance"].as_str().unwrap().into(),
            )
        })
        .collect()
}

#[test]
fn instantane_du_dump() {
    let f = copie_fixture("rust-mini");
    let (_, dump) = indexer(f.path());
    let texte = serde_json::to_string_pretty(&dump).unwrap() + "\n";
    let chemin = racine_depot().join("tests/fixtures/rust-mini.dump.json");
    if std::env::var_os("WIKIK_BENIR").is_some() || !chemin.exists() {
        std::fs::write(&chemin, &texte).unwrap();
    }
    let attendu = std::fs::read_to_string(&chemin).unwrap();
    assert!(
        attendu == texte,
        "le dump diffère de l'instantané ; relancer avec WIKIK_BENIR=1 si le changement est voulu"
    );
}

#[test]
fn deterministe_sur_deux_emplacements() {
    let a = copie_fixture("rust-mini");
    let b = copie_fixture("rust-mini");
    assert_eq!(indexer(a.path()).1, indexer(b.path()).1);
}

#[test]
fn identifiants_specs02() {
    let f = copie_fixture("rust-mini");
    let (_, d) = indexer(f.path());
    for id in [
        "src/lib.rs::entree",
        "src/geo.rs::Point::new",
        "src/geo.rs::<Point as Display>::fmt",
        "src/util.rs::interne::profond",
        "src/util.rs::exe@cfg(windows)",
        "src/util.rs::exe@cfg(not(windows))",
    ] {
        symbole(&d, id);
    }
    assert_eq!(symbole(&d, "src/util.rs::tests::ok")["est_test"], 1);
    assert_eq!(
        symbole(&d, "src/geo.rs::<Point as Display>::fmt")["visibilite"],
        "pub"
    );
}

#[test]
fn resolution_des_appels() {
    let f = copie_fixture("rust-mini");
    let (_, d) = indexer(f.path());
    let appels = aretes(&d, "appel");
    let attendu = |s: &str, c: &str, conf: &str| {
        assert!(
            appels.contains(&(s.into(), c.into(), conf.into())),
            "arête manquante {s} -> {c} ({conf}) dans {appels:#?}"
        )
    };
    attendu("src/lib.rs::entree", "src/lib.rs::aide", "certain"); // direct local
    attendu("src/lib.rs::entree", "src/geo.rs::aire", "certain"); // direct importé
    attendu("src/lib.rs::entree", "src/util.rs::double", "certain");
    attendu("src/lib.rs::entree", "src/geo.rs::Point::new", "certain"); // chemin
    attendu("src/lib.rs::entree", "src/geo.rs::Point::norme", "probable"); // méthode, nom unique
    attendu(
        "src/geo.rs::Point::norme",
        "src/geo.rs::Point::carre",
        "certain",
    ); // self.f()
    attendu(
        "src/util.rs::interne::profond",
        "src/util.rs::double",
        "certain",
    ); // super::
    attendu(
        "src/util.rs::double",
        "src/util.rs::exe@cfg(windows)",
        "certain",
    ); // variantes cfg
    attendu(
        "src/util.rs::double",
        "src/util.rs::exe@cfg(not(windows))",
        "certain",
    );
}

#[test]
fn balises_et_liens() {
    let f = copie_fixture("rust-mini");
    let (_, d) = indexer(f.path());
    assert!(aretes(&d, "req").contains(&(
        "src/lib.rs::entree".into(),
        "EF-001".into(),
        "certain".into()
    )));
    assert!(aretes(&d, "adr").contains(&(
        "src/geo.rs::Point::norme".into(),
        "ADR-0001".into(),
        "certain".into()
    )));
    let casses: Vec<&str> = d["liens_doc"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l["cible"].is_null())
        .map(|l| l["cible_brute"].as_str().unwrap())
        .collect();
    assert_eq!(casses, vec!["page-absente", "B-404"]);
}

#[test]
fn incremental_egal_reconstruction() {
    let f = copie_fixture("rust-mini");
    let (r1, _) = indexer(f.path());
    assert_eq!(r1.analyses, 8);
    let (r2, _) = indexer(f.path());
    assert_eq!(r2.analyses, 0, "rien n'a changé");

    let geo = f.path().join("src/geo.rs");
    let texte = std::fs::read_to_string(&geo)
        .unwrap()
        .replace("cote * cote", "cote * cote * 1.0");
    std::fs::write(&geo, texte).unwrap();
    std::fs::remove_file(f.path().join("docs/projet/notes.md")).unwrap();
    let (r3, incremental) = indexer(f.path());
    assert_eq!((r3.analyses, r3.supprimes), (1, 1));

    std::fs::remove_dir_all(f.path().join(".wikik")).unwrap();
    let (_, complet) = indexer(f.path());
    assert_eq!(incremental, complet);
}

#[test]
fn hash_insensible_au_reformatage() {
    let f = copie_fixture("rust-mini");
    let (_, avant) = indexer(f.path());
    let statut = std::process::Command::new("rustfmt")
        .args([
            "--edition",
            "2024",
            "--config",
            "max_width=50,use_small_heuristics=Max",
        ])
        .args(["src/lib.rs", "src/geo.rs", "src/util.rs"])
        .current_dir(f.path())
        .status();
    let Ok(statut) = statut else {
        eprintln!("rustfmt absent : test ignoré");
        return;
    };
    assert!(statut.success());
    let (_, apres) = indexer(f.path());
    for s in avant["symboles"].as_array().unwrap() {
        let id = s["id"].as_str().unwrap();
        assert_eq!(
            s["hash_corps"],
            symbole(&apres, id)["hash_corps"],
            "hash_corps de {id}"
        );
        assert_eq!(
            s["hash_signature"],
            symbole(&apres, id)["hash_signature"],
            "hash_signature de {id}"
        );
    }
}

#[test]
fn erreur_de_parse_non_bloquante() {
    let f = copie_fixture("rust-mini");
    std::fs::write(
        f.path().join("src/casse.rs"),
        "/// Doc.\npub fn ok() {}\nfn casse( {\n",
    )
    .unwrap();
    let (_, d) = indexer(f.path());
    let fichier = d["fichiers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["chemin"] == "src/casse.rs")
        .unwrap();
    assert!(fichier["erreurs_parse"].as_i64().unwrap() > 0);
    symbole(&d, "src/casse.rs::ok");
}

#[test]
fn exclusions_de_la_config() {
    let f = copie_fixture("rust-mini");
    std::fs::write(
        f.path().join("wikik.toml"),
        "[index]\nexclure = [\"src/util.rs\"]\n",
    )
    .unwrap();
    let (_, d) = indexer(f.path());
    assert!(
        d["fichiers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["chemin"] != "src/util.rs")
    );
}
