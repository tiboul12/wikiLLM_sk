//! `wikik reception` : carte scannée au lancement, remarques avec images, images servies sans risque,
//! traitement, outil de requête (SPEC-22, SPEC-23).

use serde_json::Value;
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
            "reception",
            "--factice",
            "--port",
            &port.to_string(),
        ])
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let limite = Instant::now() + Duration::from_secs(20);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(Instant::now() < limite, "le cahier ne démarre pas");
        std::thread::sleep(Duration::from_millis(50));
    }
    Serveur(enfant, port)
}

/// Requête HTTP/1.1 minimale : (code, en-têtes, corps).
fn http(port: u16, methode: &str, chemin: &str, corps: Option<&str>) -> (u16, String, Vec<u8>) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let corps = corps.unwrap_or("");
    let r = format!(
        "{methode} {chemin} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{corps}",
        corps.len()
    );
    s.write_all(r.as_bytes()).unwrap();
    let mut t = vec![];
    s.read_to_end(&mut t).unwrap();
    let fin = t.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let entetes = String::from_utf8_lossy(&t[..fin]).to_string();
    let code = entetes[9..12].parse().unwrap();
    let mut corps = t[fin + 4..].to_vec();
    if entetes
        .to_lowercase()
        .contains("transfer-encoding: chunked")
    {
        // Un seul morceau suffit ici : « taille\r\n données \r\n0 ».
        let p = corps.windows(2).position(|w| w == b"\r\n").unwrap();
        let n =
            usize::from_str_radix(std::str::from_utf8(&corps[..p]).unwrap().trim(), 16).unwrap();
        corps = corps[p + 2..p + 2 + n].to_vec();
    }
    (code, entetes, corps)
}

fn json(port: u16, methode: &str, chemin: &str, corps: Option<&str>) -> (u16, Value) {
    let (c, _, b) = http(port, methode, chemin, corps);
    (c, serde_json::from_slice(&b).unwrap_or(Value::Null))
}

/// PNG minimal valide (1 × 1), en data: URL.
const PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

/// @req EF-120, EF-123, EF-124, EF-125
#[test]
fn cahier_de_bout_en_bout() {
    let d = tempfile::tempdir().unwrap();
    copier(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/reception-mini"),
        d.path(),
    );
    let s = demarrer(d.path());
    let p = s.1;

    // La carte est construite au premier lancement.
    let (c, e) = json(p, "GET", "/api/reception/etat", None);
    assert_eq!(c, 200);
    let cles: Vec<&str> = e["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["cle"].as_str().unwrap())
        .collect();
    assert!(
        cles.contains(&"general") && cles.contains(&"/panier") && cles.contains(&"bibliotheque"),
        "{cles:?}"
    );
    assert_eq!(json(p, "GET", "/api/mode", None).1["reception"], true);

    // Une remarque avec deux captures.
    let corps = serde_json::json!({ "page": "/panier", "texte": "Le total ne se met pas à jour\nquand on retire un article.", "genre": "bug", "gravite": "bloquant", "images": [PNG, PNG] }).to_string();
    assert_eq!(
        json(p, "POST", "/api/reception/remarque", Some(&corps)),
        (200, Value::String("REC-001".into()))
    );
    let fichier =
        std::fs::read_to_string(d.path().join("docs/reception/remarques/REC-001.md")).unwrap();
    assert!(
        fichier.contains("![capture 2](../images/REC-001-2.png)")
            && fichier.contains(r#"symboles: ["react/src/App.tsx::Panier"]"#),
        "{fichier}"
    );
    // Image servie ; SVG déguisé, chemin hors zone ou remontée refusés.
    let (c, entetes, octets) = http(
        p,
        "GET",
        "/api/image?chemin=docs/reception/images/REC-001-1.png",
        None,
    );
    assert!(
        c == 200 && entetes.to_lowercase().contains("image/png") && octets.starts_with(b"\x89PNG"),
        "{c} {entetes}"
    );
    std::fs::write(
        d.path().join("docs/reception/images/x.png"),
        "<svg onload=alert(1)>",
    )
    .unwrap();
    assert_eq!(
        http(
            p,
            "GET",
            "/api/image?chemin=docs/reception/images/x.png",
            None
        )
        .0,
        404
    );
    assert_eq!(
        http(p, "GET", "/api/image?chemin=react/src/App.tsx", None).0,
        404
    );
    assert_eq!(
        http(
            p,
            "GET",
            "/api/image?chemin=docs/reception/../../etc/passwd",
            None
        )
        .0,
        404
    );
    let svg = serde_json::json!({ "page": "general", "texte": "x", "genre": "bug", "gravite": "detail", "images": ["data:image/png;base64,PHN2Zz4="] }).to_string();
    assert_eq!(
        json(p, "POST", "/api/reception/remarque", Some(&svg)).0,
        400
    );

    // L'agent la lit avec ce qu'il faut pour agir, puis la traite.
    let q = Command::new(env!("CARGO_BIN_EXE_wikik"))
        .args(["--racine", d.path().to_str().unwrap(), "query", "reception"])
        .output()
        .unwrap();
    let q = String::from_utf8_lossy(&q.stdout);
    assert!(
        q.contains("REC-001 · bloquant · bug · page « Panier »")
            && q.contains("code : react/src/App.tsx::Panier")
            && q.contains("REC-001-2.png"),
        "{q}"
    );
    let t = serde_json::json!({ "id": "REC-001", "statut": "traitee", "resolution": "Total recalculé au retrait.", "commit": "abc1234" }).to_string();
    assert_eq!(json(p, "POST", "/api/reception/traiter", Some(&t)).0, 200);
    let (_, e) = json(p, "GET", "/api/reception/etat", None);
    assert_eq!(e["remarques"][0]["statut"], "traitee");
    assert_eq!(
        e["remarques"][0]["texte"],
        "Le total ne se met pas à jour\nquand on retire un article."
    );
    assert!(
        std::fs::read_to_string(d.path().join("docs/reception/remarques/REC-001.md"))
            .unwrap()
            .contains("## Traitement\n\n**traitee** : Total recalculé au retrait.")
    );
    // Modifier (nouvelle version), puis supprimer toute la chaîne.
    let m = serde_json::json!({ "id": "REC-001", "page": "/panier", "texte": "Total faux au retrait", "genre": "bug", "gravite": "genant", "garder": ["docs/reception/images/REC-001-1.png"] }).to_string();
    assert_eq!(
        json(p, "POST", "/api/reception/remarque", Some(&m)),
        (200, Value::String("REC-001.2".into()))
    );
    let (_, e) = json(p, "GET", "/api/reception/etat", None);
    let v2 = e["remarques"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "REC-001.2")
        .unwrap()
        .clone();
    assert_eq!(
        (
            v2["statut"].as_str(),
            v2["historique"].as_array().unwrap().len()
        ),
        (Some("traitee"), 2)
    );
    assert_eq!(
        json(
            p,
            "POST",
            "/api/reception/application",
            Some(r#"{"url":"http://10.0.0.21:3000"}"#)
        )
        .0,
        200
    );
    assert_eq!(
        json(p, "GET", "/api/reception/etat", None).1["url_application"],
        "http://10.0.0.21:3000"
    );
    let (c, sup) = json(
        p,
        "POST",
        "/api/reception/supprimer",
        Some(r#"{"id":"REC-001.2"}"#),
    );
    assert_eq!((c, sup), (200, serde_json::json!(["REC-001", "REC-001.2"])));
    assert!(
        !d.path()
            .join("docs/reception/images/REC-001-1.png")
            .exists()
    );
    // Le wiki voit les remarques (zone réception).
    let r2 = serde_json::json!({ "page": "general", "texte": "Remarque générale", "genre": "question", "gravite": "detail" }).to_string();
    // Un numéro n'est jamais réutilisé, même après suppression.
    assert_eq!(
        json(p, "POST", "/api/reception/remarque", Some(&r2)).1,
        "REC-002"
    );
    let (_, projet) = json(p, "GET", "/api/projet", None);
    assert!(
        projet["documents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["id"] == "REC-002" && x["zone"] == "reception")
    );
}
