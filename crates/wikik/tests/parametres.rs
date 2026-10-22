//! Paramètres et accès du propriétaire (EF-147, ADR-0013) : lecture et écriture de wikik.toml par le
//! serveur, clé du lancement posée en cookie.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

struct Serveur(Child, u16);

impl Drop for Serveur {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

fn demarrer(racine: &Path) -> Serveur {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let enfant = Command::new(env!("CARGO_BIN_EXE_wikik"))
        .args([
            "--racine",
            racine.to_str().unwrap(),
            "--factice",
            "--port",
            &port.to_string(),
        ])
        // Au premier plan : le test arrête le serveur en tuant ce processus.
        .env("WIKIK_PREMIER_PLAN", "1")
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let limite = Instant::now() + Duration::from_secs(20);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(Instant::now() < limite, "le serveur ne démarre pas");
        std::thread::sleep(Duration::from_millis(50));
    }
    Serveur(enfant, port)
}

/// Requête HTTP/1.1 minimale : (code, en-têtes, corps).
fn http(port: u16, methode: &str, chemin: &str, corps: &str) -> (u16, String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let r = format!(
        "{methode} {chemin} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{corps}",
        corps.len()
    );
    s.write_all(r.as_bytes()).unwrap();
    let mut t = vec![];
    s.read_to_end(&mut t).unwrap();
    let t = String::from_utf8_lossy(&t).to_string();
    let (entetes, corps) = t.split_once("\r\n\r\n").unwrap();
    (
        entetes[9..12].parse().unwrap(),
        entetes.to_string(),
        corps.to_string(),
    )
}

/// @req EF-147
#[test]
fn parametres_et_cle() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(
        d.path().join("wikik.toml"),
        "# commentaire gardé\n[serveur]\nreseau = false\n",
    )
    .unwrap();
    let s = demarrer(d.path());
    let p = s.1;

    let (c, _, corps) = http(p, "GET", "/api/parametres", "");
    assert_eq!(c, 200);
    let mut params: serde_json::Value = serde_json::from_str(&corps).unwrap();
    assert_eq!(
        params["proprietaire"], true,
        "la machine locale est propriétaire"
    );
    assert_eq!(params["factice"], true);
    let mut p2 = params["parametres"].take();
    p2["modele"] = "sonnet".into();
    let (c, _, _) = http(p, "POST", "/api/parametres", &p2.to_string());
    assert_eq!(c, 200);
    let toml = std::fs::read_to_string(d.path().join("wikik.toml")).unwrap();
    assert!(
        toml.starts_with("# commentaire gardé") && toml.contains("modele = \"sonnet\""),
        "{toml}"
    );

    // Un skill modifié est écrit dans .claude/skills du projet, puis retiré par « revenir à l'origine ».
    let corps =
        r#"{"nom":"wikik-lecture","texte":"---\nname: wikik-lecture\ndescription: x\n---\nLis."}"#;
    let (c, _, _) = http(p, "POST", "/api/skills", corps);
    assert_eq!(c, 200);
    let skill = d.path().join(".claude/skills/wikik-lecture/SKILL.md");
    assert!(std::fs::read_to_string(&skill).unwrap().ends_with("Lis.\n"));
    let (c, _, _) = http(
        p,
        "POST",
        "/api/skills/retablir",
        r#"{"nom":"wikik-lecture"}"#,
    );
    assert_eq!(c, 200);
    assert!(!skill.exists());

    // La clé du lancement pose le cookie ; une fausse clé ne pose rien.
    let cle = std::fs::read_to_string(d.path().join(".wikik/cle")).unwrap();
    let (c, entetes, _) = http(p, "GET", &format!("/?cle={cle}"), "");
    assert_eq!(c, 303);
    assert!(
        entetes.contains(&format!("wikik_cle={cle}")) && entetes.contains("HttpOnly"),
        "{entetes}"
    );
    let (c, entetes, _) = http(p, "GET", "/?cle=0123", "");
    assert!(c == 200 && !entetes.contains("wikik_cle"), "{entetes}");
}

/// Sans `WIKIK_PREMIER_PLAN`, `wikik` rend la main et le serveur tourne détaché ; relancé, il le
/// signale ; le bouton « Arrêter le serveur » l'arrête (ADR-0014).
///
/// @req EF-130
#[test]
fn serveur_detache_et_arret() {
    let d = tempfile::tempdir().unwrap();
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let lancer = || {
        Command::new(env!("CARGO_BIN_EXE_wikik"))
            .args([
                "--racine",
                d.path().to_str().unwrap(),
                "--factice",
                "--port",
                &port.to_string(),
            ])
            .output()
            .unwrap()
    };
    let o = lancer();
    let texte = String::from_utf8_lossy(&o.stderr).to_string();
    assert!(
        o.status.success() && texte.contains("arrière-plan") && texte.contains(&port.to_string()),
        "{texte}"
    );
    assert!(
        TcpStream::connect(("127.0.0.1", port)).is_ok(),
        "le serveur tourne après le retour de wikik"
    );
    let o = lancer();
    assert!(String::from_utf8_lossy(&o.stderr).contains("tourne déjà"));
    let (c, _, _) = http(port, "POST", "/api/serveur/arreter", "{}");
    assert_eq!(c, 200);
    let limite = Instant::now() + Duration::from_secs(8);
    // Le port se ferme avant que le serveur ne retire `serveur.json` : on attend les deux.
    while TcpStream::connect(("127.0.0.1", port)).is_ok()
        || d.path().join(".wikik/serveur.json").exists()
    {
        assert!(Instant::now() < limite, "le serveur ne s'arrête pas");
        std::thread::sleep(Duration::from_millis(100));
    }
}
