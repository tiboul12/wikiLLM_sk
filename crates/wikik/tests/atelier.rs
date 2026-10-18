//! `wikik atelier --factice` : routes de l'atelier, garde d'origine, lecture du wiki (SPEC-18, SPEC-20).

use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

struct Serveur(Child, u16);

impl Drop for Serveur {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

fn demarrer(racine: &std::path::Path) -> Serveur {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let enfant = Command::new(env!("CARGO_BIN_EXE_wikik"))
        .args([
            "--racine",
            racine.to_str().unwrap(),
            "atelier",
            "--factice",
            "--port",
            &port.to_string(),
        ])
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let limite = Instant::now() + Duration::from_secs(20);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(Instant::now() < limite, "l'atelier ne démarre pas");
        std::thread::sleep(Duration::from_millis(50));
    }
    Serveur(enfant, port)
}

/// Requête HTTP/1.1 minimale : (code, corps).
fn http(
    port: u16,
    methode: &str,
    chemin: &str,
    corps: Option<&str>,
    entetes: &[&str],
) -> (u16, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let corps = corps.unwrap_or("");
    let mut r = format!(
        "{methode} {chemin} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Length: {}\r\n",
        corps.len()
    );
    for e in entetes {
        r.push_str(&format!("{e}\r\n"));
    }
    r.push_str("\r\n");
    r.push_str(corps);
    s.write_all(r.as_bytes()).unwrap();
    let mut t = String::new();
    s.read_to_string(&mut t).unwrap();
    let code = t[9..12].parse().unwrap();
    let corps = t
        .split_once("\r\n\r\n")
        .map(|x| x.1.to_string())
        .unwrap_or_default();
    // Corps éventuellement découpé (chunked) : on garde la ligne JSON.
    let corps = corps
        .lines()
        .find(|l| l.starts_with('{') || l.starts_with('[') || l.starts_with('"'))
        .unwrap_or(&corps)
        .to_string();
    (code, corps)
}

fn post(port: u16, route: &str, json: &str) -> (u16, String) {
    http(
        port,
        "POST",
        &format!("/api/atelier/{route}"),
        Some(json),
        &["Content-Type: application/json"],
    )
}

/// @req EF-100, EF-104
#[test]
fn atelier_factice_de_bout_en_bout() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("docs/projet")).unwrap();
    let s = demarrer(d.path());
    let p = s.1;

    let (c, _) = post(
        p,
        "proposer",
        r#"{"etape":"pitch","sous_etape":"pitch","entree":"Une appli de randonnée."}"#,
    );
    assert_eq!(c, 200);
    assert_eq!(
        post(
            p,
            "decider",
            r#"{"etape":"pitch","sous_etape":"pitch","indice":0,"decision":"garder"}"#
        ),
        (200, "\"PITCH\"".into())
    );
    let (c, e) = post(
        p,
        "decider",
        r#"{"etape":"pitch","sous_etape":"pitch","indice":0,"decision":"garder"}"#,
    );
    assert!(c == 400 && e.contains("déjà décidée"), "{c} {e}");

    // Une page tierce ne peut pas écrire ; un corps non JSON est refusé.
    let (c, _) = http(
        p,
        "POST",
        "/api/atelier/piloter",
        Some(r#"{"etape":"pitch","action":"terminer"}"#),
        &[
            "Content-Type: application/json",
            "Origin: http://ailleurs.example",
        ],
    );
    assert_eq!(c, 403);
    let (c, _) = http(
        p,
        "POST",
        "/api/atelier/piloter",
        Some(r#"{"etape":"pitch","action":"terminer"}"#),
        &["Content-Type: text/plain"],
    );
    assert_eq!(c, 415);
    assert_eq!(
        post(p, "piloter", r#"{"etape":"pitch","action":"terminer"}"#).0,
        200
    );

    let (c, e) = http(p, "GET", "/api/atelier/etat", None, &[]);
    assert_eq!(c, 200);
    let e: Value = serde_json::from_str(&e).unwrap();
    assert_eq!(e["fournisseur"], "factice");
    assert_eq!(e["etapes"][0]["etat"], "faite");
    assert_eq!(e["etapes"][0]["artefacts"][0]["id"], "PITCH");

    // L'API du wiki est servie aussi, et voit l'atelier.
    let (c, projet) = http(p, "GET", "/api/projet", None, &[]);
    assert!(
        c == 200 && projet.contains("\"zone\":\"atelier\""),
        "{projet}"
    );
    assert!(d.path().join(".wikik/atelier/journal.jsonl").exists());

    // Le front sait qu'il est dans l'atelier ; le wiki connaît l'adresse de l'atelier (SPEC-21 R1).
    assert_eq!(
        http(p, "GET", "/api/mode", None, &[]),
        (200, r#"{"atelier":true,"reception":false}"#.into())
    );
    // Port réellement pris, noté dans .wikik/serveurs.json (plusieurs projets ouverts à la fois).
    assert!(
        projet.contains(&format!(r#""url_atelier":"http://127.0.0.1:{p}""#)),
        "{projet}"
    );
    let serveurs = std::fs::read_to_string(d.path().join(".wikik/serveurs.json")).unwrap();
    assert_eq!(serveurs, format!(r#"{{"atelier":{p}}}"#));
}
