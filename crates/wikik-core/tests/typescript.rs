//! Critères d'acceptation de SPEC-15 sur la fixture `tests/fixtures/ts-mini`.

use serde_json::Value;
use std::path::Path;
use wikik_core::Index;

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

fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    copier(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ts-mini"),
        d.path(),
    );
    d
}

fn dump(racine: &Path) -> Value {
    let mut i = Index::ouvrir(racine).unwrap();
    i.mettre_a_jour().unwrap();
    i.dump_json().unwrap()
}

fn symbole<'a>(d: &'a Value, id: &str) -> &'a Value {
    d["symboles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == id)
        .unwrap_or_else(|| panic!("absent : {id}"))
}

/// Symboles, genres, visibilités, résumés, en-tête et fichiers ignorés.
///
/// @req EF-070, EF-073
#[test]
fn symboles_typescript() {
    let f = fixture();
    let d = dump(f.path());
    for (id, genre, vis) in [
        ("src/lib/calcul.ts::doubler", "fn", "pub"),
        ("src/lib/calcul.ts::tripler", "fn", "pub"),
        ("src/lib/calcul.ts::interne", "fn", "prive"),
        ("src/lib/calcul.ts::Forme", "interface", "pub"),
        ("src/lib/calcul.ts::Couleur", "type", "pub"),
        ("src/lib/calcul.ts::Unite", "enum", "pub"),
        ("src/composant.ts::Composant", "composant", "pub"),
        ("src/composant.ts::Composant::valeur", "champ", "pub"),
        ("src/composant.ts::Composant::cache", "champ", "prive"),
        ("src/composant.ts::Composant::afficher", "methode", "pub"),
        ("src/composant.ts::Composant::calculer", "methode", "prive"),
    ] {
        let s = symbole(&d, id);
        assert_eq!(
            (
                s["genre"].as_str().unwrap(),
                s["visibilite"].as_str().unwrap()
            ),
            (genre, vis),
            "{id}"
        );
    }
    assert_eq!(
        symbole(&d, "src/lib/calcul.ts::doubler")["resume"],
        "Double une valeur ; sature à 1000."
    );
    assert_eq!(symbole(&d, "src/app.spec.ts::testTotal")["est_test"], 1);
    let calcul = d["fichiers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["chemin"] == "src/lib/calcul.ts")
        .unwrap();
    assert_eq!(
        calcul["entete"],
        "Fonctions de calcul partagées.\nSert de fixture TypeScript (SPEC-15)."
    );
    assert!(
        d["fichiers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["chemin"] != "src/types.d.ts"),
        ".d.ts ignoré"
    );
}

/// Imports nommés, renommés, `* as`, `new`, `this.f()`.
///
/// @req EF-071
#[test]
fn resolution_typescript() {
    let f = fixture();
    let d = dump(f.path());
    let aretes: Vec<(String, String, String)> = d["aretes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["genre"] == "appel")
        .map(|a| {
            (
                a["source"].as_str().unwrap().into(),
                a["cible"].as_str().unwrap().into(),
                a["confiance"].as_str().unwrap().into(),
            )
        })
        .collect();
    let attendu = |s: &str, c: &str, conf: &str| {
        assert!(
            aretes.contains(&(s.into(), c.into(), conf.into())),
            "{s} -> {c} ({conf}) absent de {aretes:#?}"
        )
    };
    attendu("src/app.ts::total", "src/lib/calcul.ts::doubler", "certain");
    attendu("src/app.ts::total", "src/lib/calcul.ts::tripler", "certain");
    attendu(
        "src/app.ts::total",
        "src/composant.ts::Composant",
        "certain",
    );
    attendu(
        "src/app.ts::total",
        "src/composant.ts::Composant::afficher",
        "probable",
    );
    attendu(
        "src/composant.ts::Composant::afficher",
        "src/composant.ts::Composant::calculer",
        "certain",
    );
    attendu("src/app.spec.ts::testTotal", "src/app.ts::total", "certain");
    attendu(
        "src/composant.ts::Composant::service",
        "src/composant.ts::Service",
        "certain",
    ); // inject(Service)
}

/// Reformatage façon Prettier : points-virgules, guillemets, virgules finales.
///
/// @req EF-072
#[test]
fn hash_insensible_au_formatage() {
    let f = fixture();
    let avant = dump(f.path());
    for fichier in ["src/lib/calcul.ts", "src/app.ts", "src/composant.ts"] {
        let p = f.path().join(fichier);
        let t = std::fs::read_to_string(&p).unwrap();
        let t = t
            .lines()
            .map(|l| l.trim_end().trim_end_matches(';').to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .replace('"', "'")
            .replace("(x: number): number {", "(\n  x: number,\n): number {");
        std::fs::write(&p, t).unwrap();
    }
    let apres = dump(f.path());
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

/// `@param`, `@returns` : aucune balise inconnue ; un champ sans doc n'est pas exigé.
///
/// @req EF-074
#[test]
fn check_typescript() {
    let f = fixture();
    let mut i = Index::ouvrir(f.path()).unwrap();
    i.mettre_a_jour().unwrap();
    let r = wikik_core::check::verifier(&i).unwrap();
    let codes: Vec<(&str, &str)> = r
        .diagnostics
        .iter()
        .map(|d| (d.code.as_str(), d.symbole.as_deref().unwrap_or("")))
        .collect();
    assert!(
        !codes.iter().any(|(c, _)| *c == "balise-inconnue"),
        "{codes:?}"
    );
    assert!(
        !codes
            .iter()
            .any(|(_, s)| s.ends_with("::cache") || s.ends_with("::interne")),
        "{codes:?}"
    );
}
