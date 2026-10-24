//! Critères de clôture de la v1 vérifiés sur le dépôt lui-même : skill complet, migration faite,
//! et wikiK passe son propre `check` sans erreur.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Racine du dépôt wikiK.
fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Le skill décrit chaque procédure de SPEC-11 et ne cite que des commandes qui existent.
///
/// @req EF-030, EF-031, EF-032
#[test]
fn skill_complet() {
    let skill = racine().join("skill");
    let texte = std::fs::read_to_string(skill.join("SKILL.md")).unwrap();
    assert!(
        texte.starts_with("---\nname: wikik\n"),
        "frontmatter du skill"
    );
    for p in ["interroger", "ecrire-bloc", "amorcer", "adr", "majeure"] {
        assert!(
            skill.join(format!("procedures/{p}.md")).exists(),
            "procédure {p}"
        );
        assert!(
            texte.contains(&format!("procedures/{p}.md")),
            "SKILL.md renvoie à {p}"
        );
    }
    let aide = Command::new(env!("CARGO_BIN_EXE_wikik"))
        .args(["query", "--help"])
        .output()
        .unwrap();
    let aide = String::from_utf8_lossy(&aide.stdout);
    for r in [
        "carte", "fichier", "symbole", "impact", "perimes", "cherche", "adr", "exigence", "ref",
    ] {
        assert!(
            texte.contains(&format!("wikik query {r}")),
            "SKILL.md documente query {r}"
        );
        assert!(aide.contains(r), "la CLI fournit query {r}");
    }
}

/// La v0 est retirée (wiki/ et legacy/) et wikiK passe son propre `check` sans erreur.
///
/// @req EF-050
#[test]
fn depot_migre_et_sans_erreur() {
    let r = racine();
    assert!(
        !r.join("wiki").exists() && !r.join("legacy").exists(),
        "wiki/ et legacy/ doivent avoir disparu"
    );
    let sortie = Command::new(env!("CARGO_BIN_EXE_wikik"))
        .args(["--racine", r.to_str().unwrap(), "check", "--format", "json"])
        .output()
        .unwrap();
    let rapport: serde_json::Value = serde_json::from_slice(&sortie.stdout).unwrap();
    assert_eq!(
        rapport["resume"]["erreurs"],
        0,
        "{}",
        serde_json::to_string_pretty(&rapport["diagnostics"]).unwrap()
    );
}
