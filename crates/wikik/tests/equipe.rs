//! Critères d'acceptation de SPEC-16 (équipe) sur la fixture `rust-mini`, via le binaire.

use std::path::Path;
use std::process::Command;

/// Copie récursive d'un dossier de fixtures dans un dossier temporaire, pour que le test écrive sans toucher au dépôt.
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

/// Copie de la fixture `rust-mini`.
fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    copier(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/rust-mini"),
        d.path(),
    );
    d
}

/// Lance le binaire `wikik` sur la copie avec ces arguments.
fn wikik(racine: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_wikik"))
        .arg("--racine")
        .arg(racine)
        .args(args)
        .output()
        .unwrap()
}

/// L'export contient front et données, et deux exports sont identiques.
///
/// @req EF-080, ENF-002
#[test]
fn export_statique_deterministe() {
    let f = fixture();
    let (a, b) = (f.path().join("site-a"), f.path().join("site-b"));
    assert!(
        wikik(f.path(), &["export", a.to_str().unwrap()])
            .status
            .success()
    );
    assert!(
        wikik(f.path(), &["export", b.to_str().unwrap()])
            .status
            .success()
    );
    for fichier in [
        "index.html",
        "donnees/projet.json",
        "donnees/check.json",
        "donnees/recherche.json",
        "donnees/graphe-complet.json",
        "donnees/fichiers/src/geo.rs.json",
        "donnees/documents/EF-001.json",
    ] {
        assert!(a.join(fichier).exists(), "{fichier} absent");
        assert_eq!(
            std::fs::read(a.join(fichier)).unwrap(),
            std::fs::read(b.join(fichier)).unwrap(),
            "{fichier} diffère"
        );
    }
    let html = std::fs::read_to_string(a.join("index.html")).unwrap();
    assert!(html.contains("wikik-statique"));
}

/// Annotations GitHub : une ligne `::error` pour la référence cassée, code de retour 1.
///
/// @req EF-081
#[test]
fn annotations_github() {
    let f = fixture();
    let s = wikik(f.path(), &["check", "--format", "github"]);
    let sortie = String::from_utf8_lossy(&s.stdout);
    assert!(
        sortie.contains("::error file=src/geo.rs,line=39,title=ref-req::"),
        "{sortie}"
    );
    assert_eq!(s.status.code(), Some(1));
}

/// `init --ci` écrit le workflow ; `init --hook` installe un hook qui bloque un commit cassé.
///
/// @req EF-082, EF-083
#[test]
fn ci_et_hook() {
    let f = fixture();
    std::fs::write(
        f.path().join("wikik.toml"),
        "[equipe]\nvalidateurs = [\"@tibo\"]\n",
    )
    .unwrap();
    let git = |args: &[&str]| {
        Command::new("git")
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(f.path())
            .output()
            .unwrap()
    };
    git(&["init", "-q"]);
    assert!(
        wikik(f.path(), &["init", "--ci", "--hook"])
            .status
            .success()
    );
    assert!(f.path().join(".github/workflows/wikik.yml").exists());
    assert!(
        std::fs::read_to_string(f.path().join(".github/CODEOWNERS"))
            .unwrap()
            .contains("@tibo")
    );
    // Le hook appelle `wikik` : on le met dans le PATH du commit.
    let bin = Path::new(env!("CARGO_BIN_EXE_wikik"))
        .parent()
        .unwrap()
        .to_path_buf();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    git(&["add", "-A"]);
    let commit = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-m",
            "x",
        ])
        .current_dir(f.path())
        .env("PATH", path)
        .output()
        .unwrap();
    assert!(
        !commit.status.success(),
        "la fixture contient `@req EF-999` : le hook doit refuser le commit"
    );
}

/// `nouveau` crée le prochain identifiant ; `stats` donne des chiffres cohérents.
///
/// @req EF-084, EF-085
#[test]
fn nouveau_et_stats() {
    let f = fixture();
    let stats = wikik(f.path(), &["stats", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_slice(&stats.stdout).unwrap();
    assert_eq!(v["fichiers"], 3);
    assert_eq!(v["exigences_must"], 2);
    assert_eq!(v["exigences_must_couvertes"], 1);
    assert!(v["documentes"].as_u64().unwrap() <= v["symboles"].as_u64().unwrap());
    let s = wikik(f.path(), &["nouveau", "exigence", "Tripler"]);
    assert_eq!(
        String::from_utf8_lossy(&s.stdout).trim(),
        "docs/projet/exigences/EF-003.md"
    );
}
