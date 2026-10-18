//! Critère d'acceptation de SPEC-14 : un client MCP obtient les outils et la même sortie que la CLI.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

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

/// Poignée de main, liste d'outils, appel, méthode inconnue, erreur d'outil.
///
/// @req EF-060
#[test]
fn dialogue_mcp() {
    let d = tempfile::tempdir().unwrap();
    copier(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/rust-mini"),
        d.path(),
    );
    let bin = env!("CARGO_BIN_EXE_wikik");
    let mut serveur = Command::new(bin)
        .args(["--racine", d.path().to_str().unwrap(), "mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut entree = serveur.stdin.take().unwrap();
    for m in [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"symbole","arguments":{"id":"norme"}}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"inconnue"}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"symbole","arguments":{}}}"#,
    ] {
        writeln!(entree, "{m}").unwrap();
    }
    drop(entree);
    let reponses: Vec<serde_json::Value> = BufReader::new(serveur.stdout.take().unwrap())
        .lines()
        .map(|l| serde_json::from_str(&l.unwrap()).unwrap())
        .collect();
    serveur.wait().unwrap();

    assert_eq!(
        reponses.len(),
        5,
        "la notification ne reçoit pas de réponse"
    );
    assert_eq!(reponses[0]["result"]["protocolVersion"], "2025-06-18");
    let outils: Vec<&str> = reponses[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    for o in [
        "carte",
        "fichier",
        "symbole",
        "impact",
        "perimes",
        "cherche",
        "adr",
        "exigence",
        "ref",
        "check",
        "atelier",
        "document",
        "atelier_rappel",
        "pages",
        "reception",
        "reception_traiter",
    ] {
        assert!(outils.contains(&o), "outil {o}");
    }
    let cli = Command::new(bin)
        .args([
            "--racine",
            d.path().to_str().unwrap(),
            "query",
            "symbole",
            "norme",
        ])
        .output()
        .unwrap();
    assert_eq!(
        reponses[2]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
        String::from_utf8_lossy(&cli.stdout)
    );
    assert_eq!(reponses[3]["error"]["code"], -32601);
    assert_eq!(reponses[4]["result"]["isError"], true);
}
