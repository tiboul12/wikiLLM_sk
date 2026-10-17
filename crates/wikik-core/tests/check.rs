//! Scénario git de SPEC-04 : la doc est périmée si le code a changé depuis le dernier commit qui a touché son bloc.

use std::path::Path;
use std::process::Command;
use wikik_core::Index;
use wikik_core::check::{RapportCheck, verifier};

struct Depot {
    dir: tempfile::TempDir,
    horloge: u32,
}

impl Depot {
    fn nouveau() -> Depot {
        let d = Depot {
            dir: tempfile::tempdir().unwrap(),
            horloge: 0,
        };
        d.git(&["init", "-q", "-b", "main"]);
        std::fs::create_dir_all(d.chemin().join("src")).unwrap();
        std::fs::create_dir_all(d.chemin().join("docs/projet/exigences")).unwrap();
        std::fs::write(
            d.chemin().join("docs/projet/exigences/EF-001.md"),
            "---\nid: EF-001\ntype: exigence\nstatut: proposé\npriorite: must\n---\n# EF-001\n",
        )
        .unwrap();
        d
    }

    fn chemin(&self) -> &Path {
        self.dir.path()
    }

    fn git(&self, args: &[&str]) {
        let ok = Command::new("git")
            .args([
                "-c",
                "user.name=test",
                "-c",
                "user.email=test@exemple.org",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(self.chemin())
            .env(
                "GIT_AUTHOR_DATE",
                format!("2026-01-01T00:00:{:02}Z", self.horloge),
            )
            .env(
                "GIT_COMMITTER_DATE",
                format!("2026-01-01T00:00:{:02}Z", self.horloge),
            )
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    fn ecrire(&self, code: &str) {
        std::fs::write(self.chemin().join("src/lib.rs"), code).unwrap();
    }

    fn commit(&mut self, message: &str) {
        self.horloge += 1;
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }

    fn check(&self) -> RapportCheck {
        let mut i = Index::ouvrir(self.chemin()).unwrap();
        i.mettre_a_jour().unwrap();
        verifier(&i).unwrap()
    }

    fn codes(&self) -> Vec<String> {
        self.check()
            .diagnostics
            .into_iter()
            .filter(|d| d.severite != "info")
            .map(|d| d.code)
            .collect()
    }
}

const V1: &str = "//! Démo.\n\n/// Double la valeur ; sature au lieu de déborder.\n///\n/// @req EF-001\npub fn f(x: u8) -> u8 {\n    x.saturating_mul(2)\n}\n";
const V2_CORPS: &str = "//! Démo.\n\n/// Double la valeur ; sature au lieu de déborder.\n///\n/// @req EF-001\npub fn f(x: u8) -> u8 {\n    x.wrapping_mul(2)\n}\n";

#[test]
fn scenario_specs04() {
    if Command::new("git").arg("--version").output().is_err() {
        eprintln!("git absent : test ignoré");
        return;
    }
    let mut d = Depot::nouveau();
    d.ecrire(V1);
    d.commit("1 : fonction documentée");
    assert!(d.codes().is_empty(), "{:?}", d.codes());
    assert_eq!(d.check().mode_reference, "git");

    // 2 : corps modifié, bloc intact → périmé.
    d.ecrire(V2_CORPS);
    d.commit("2 : corps modifié");
    assert_eq!(d.codes(), vec!["perime-corps"]);

    // 3 : @relu rend le bloc frais.
    {
        let mut i = Index::ouvrir(d.chemin()).unwrap();
        i.mettre_a_jour().unwrap();
        wikik_core::relu::marquer(&i, "src/lib.rs::f", "2026-01-01").unwrap();
    }
    assert!(
        std::fs::read_to_string(d.chemin().join("src/lib.rs"))
            .unwrap()
            .contains("/// @relu 2026-01-01")
    );
    d.commit("3 : relu");
    assert!(d.codes().is_empty(), "{:?}", d.codes());

    // 4 : reformatage seul → rien.
    let fmt = Command::new("rustfmt")
        .args([
            "--edition",
            "2024",
            "--config",
            "max_width=40,use_small_heuristics=Max",
            "src/lib.rs",
        ])
        .current_dir(d.chemin())
        .status();
    if fmt.is_ok_and(|s| s.success()) {
        d.commit("4 : rustfmt");
        assert!(d.codes().is_empty(), "après rustfmt : {:?}", d.codes());
    }

    // 5 : corps modifié non commité → périmé ; bloc aussi modifié non commité → frais.
    let actuel = std::fs::read_to_string(d.chemin().join("src/lib.rs")).unwrap();
    d.ecrire(&actuel.replace("wrapping_mul", "checked_mul(2).unwrap_or(255).min"));
    assert_eq!(d.codes(), vec!["perime-corps"]);
    let modifie = std::fs::read_to_string(d.chemin().join("src/lib.rs")).unwrap();
    d.ecrire(&modifie.replace("sature au lieu de déborder", "sature à 255"));
    assert!(d.codes().is_empty(), "{:?}", d.codes());

    // 6 : signature changée.
    d.ecrire(V1);
    d.commit("6a : retour à V1");
    let v = std::fs::read_to_string(d.chemin().join("src/lib.rs")).unwrap();
    d.ecrire(&v.replace("pub fn f(x: u8) -> u8", "pub fn f(x: u16) -> u16"));
    d.commit("6b : signature");
    assert_eq!(d.codes(), vec!["perime-signature"]);
}

#[test]
fn references_cassees_et_json_stable() {
    let mut d = Depot::nouveau();
    d.ecrire("//! Démo.\n\n/// Fait une chose précise.\n///\n/// @req EF-999\n/// @adr 42\npub fn g() {}\n");
    d.commit("refs");
    let r = d.check();
    let codes: Vec<&str> = r.diagnostics.iter().map(|x| x.code.as_str()).collect();
    assert!(
        codes.contains(&"ref-req") && codes.contains(&"ref-adr"),
        "{codes:?}"
    );
    assert_eq!(r.code_retour(false), 1);
    let a = serde_json::to_string(&d.check()).unwrap();
    let b = serde_json::to_string(&d.check()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn severites_configurables() {
    let mut d = Depot::nouveau();
    d.ecrire("//! Démo.\n\npub fn sans_doc() {}\n");
    std::fs::write(
        d.chemin().join("wikik.toml"),
        "[check.severites]\nmanquant = \"erreur\"\n",
    )
    .unwrap();
    d.commit("config");
    let r = d.check();
    assert_eq!(r.resume.erreurs, 1);
    std::fs::write(
        d.chemin().join("wikik.toml"),
        "[check.severites]\nmanquant = \"ignore\"\n",
    )
    .unwrap();
    assert!(d.codes().is_empty());
}
