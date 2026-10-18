//! `wikik mcp` : serveur MCP sur stdio qui expose les requêtes de l'agent comme outils (SPEC-14).
//!
//! JSON-RPC 2.0, un message par ligne ; rien d'autre que les réponses n'est écrit sur stdout.
//! Les sorties des outils sont celles de `wikik query` en mode texte, au caractère près.
//!
//! @req EF-060, EF-029
//! @adr 2

use anyhow::Result;
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::Path;
use wikik_core::{Index, check, query};

/// Description d'un outil : nom, description, paramètres (nom, description, requis).
struct Outil {
    nom: &'static str,
    description: &'static str,
    parametres: &'static [(&'static str, &'static str, bool)],
}

const OUTILS: &[Outil] = &[
    Outil {
        nom: "carte",
        description: "Vue d'ensemble du projet : fichiers avec leur rôle en une ligne, documents de la zone Projet. À appeler en début de session.",
        parametres: &[],
    },
    Outil {
        nom: "fichier",
        description: "En-tête, imports et symboles (résumé, statut de doc) d'un fichier, sans son code.",
        parametres: &[(
            "chemin",
            "Chemin du fichier, complet ou suffixe unique (ex. `src/photo.rs`).",
            true,
        )],
    },
    Outil {
        nom: "symbole",
        description: "Signature, doc, appelants, appelés, ADR et exigences d'un symbole. À appeler avant de modifier une fonction.",
        parametres: &[
            (
                "id",
                "Identifiant `chemin::Type::methode`, ou nom partiel s'il est unique.",
                true,
            ),
            (
                "code",
                "true pour ajouter le code source du symbole.",
                false,
            ),
        ],
    },
    Outil {
        nom: "impact",
        description: "Appelants transitifs, décisions, exigences et pièges : ce qu'il faut relire avant de modifier un symbole.",
        parametres: &[
            ("id", "Identifiant du symbole, complet ou partiel.", true),
            (
                "profondeur",
                "Profondeur de recherche des appelants (défaut 3).",
                false,
            ),
        ],
    },
    Outil {
        nom: "perimes",
        description: "Symboles dont la doc est périmée ou manquante.",
        parametres: &[],
    },
    Outil {
        nom: "cherche",
        description: "Recherche plein texte dans les blocs de doc et les documents du projet.",
        parametres: &[("texte", "Mots à chercher.", true)],
    },
    Outil {
        nom: "adr",
        description: "Une décision d'architecture (ADR) et les symboles qui l'appliquent.",
        parametres: &[("id", "Numéro ou identifiant (`7`, `ADR-0007`).", true)],
    },
    Outil {
        nom: "exigence",
        description: "Une exigence : énoncé, besoins, critère, symboles et tests qui la citent.",
        parametres: &[("id", "Identifiant `EF-xxx` ou `ENF-xxx`.", true)],
    },
    Outil {
        nom: "ref",
        description: "Résout une référence `@wiki:` copiée depuis le wiki (symbole, fichier ou section de document).",
        parametres: &[(
            "reference",
            "La référence complète, ex. `@wiki:projet/src/a.rs::f`.",
            true,
        )],
    },
    Outil {
        nom: "atelier",
        description: "Artefacts de l'atelier de définition du besoin (pitch, idées, analyses, storyboards, clarifications…) : identifiant, titre, statut.",
        parametres: &[],
    },
    Outil {
        nom: "document",
        description: "Contenu complet d'un document de l'atelier ou du projet (frontmatter et texte), par identifiant : IDEE-003, SB-001, B-002, EF-010…",
        parametres: &[("id", "Identifiant du document.", true)],
    },
    Outil {
        nom: "atelier_rappel",
        description: "Rouvre l'atelier sur un besoin, une exigence, un artefact ou un symbole flou : crée une clarification avec la question et renvoie l'adresse de son écran, à transmettre à l'utilisateur. Seul outil qui écrit (un fichier CLAR-xxx).",
        parametres: &[
            (
                "sujet",
                "Identifiant à clarifier (B-002, EF-010, AC-019, SB-001, chemin::symbole…).",
                true,
            ),
            ("question", "Ce qui est flou, en une ou deux phrases.", true),
        ],
    },
    Outil {
        nom: "pages",
        description: "Pages (écrans) de l'application, avec leur route et le symbole qui les affiche : la carte du cahier de réception, ou ce que le scan du code trouve.",
        parametres: &[],
    },
    Outil {
        nom: "reception",
        description: "Remarques ouvertes du cahier de réception (retours de l'utilisateur, une par carte) : page, genre, gravité, texte, symboles de la page, chemins des captures (à ouvrir avec Read). Commencer par les bloquantes.",
        parametres: &[],
    },
    Outil {
        nom: "reception_traiter",
        description: "Enregistre le traitement d'une remarque de réception une fois la correction faite (ou décidée) : statut traitee, reportee ou rejetee, explication en une phrase, commit éventuel. N'écrit que dans la remarque.",
        parametres: &[
            ("id", "Identifiant de la remarque (REC-001…).", true),
            (
                "statut",
                "traitee, reportee, rejetee (ou ouverte pour rouvrir).",
                true,
            ),
            (
                "resolution",
                "Ce qui a été fait ou pourquoi c'est reporté / rejeté.",
                true,
            ),
            (
                "commit",
                "Hash court du commit qui corrige, s'il y en a un.",
                false,
            ),
        ],
    },
    Outil {
        nom: "check",
        description: "Diagnostics de doc : périmée, manquante, références cassées, exigences non couvertes.",
        parametres: &[],
    },
];

/// Boucle principale : lit stdin jusqu'à sa fermeture.
pub fn servir(racine: &Path) -> Result<()> {
    let mut index = Index::ouvrir(racine)?;
    index.mettre_a_jour()?;
    eprintln!("wikik mcp : prêt sur {}", index.racine.display());
    let entree = std::io::stdin().lock();
    let mut sortie = std::io::stdout().lock();
    for ligne in entree.lines() {
        let ligne = ligne?;
        if ligne.trim().is_empty() {
            continue;
        }
        let reponse = match serde_json::from_str::<Value>(&ligne) {
            Ok(message) => traiter(&mut index, &message),
            Err(e) => Some(erreur(Value::Null, -32700, &format!("JSON invalide : {e}"))),
        };
        if let Some(r) = reponse {
            writeln!(sortie, "{r}")?;
            sortie.flush()?;
        }
    }
    Ok(())
}

fn erreur(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Réponse à un message ; `None` pour une notification.
fn traiter(index: &mut Index, m: &Value) -> Option<Value> {
    let id = m.get("id").cloned();
    let methode = m.get("method").and_then(Value::as_str).unwrap_or_default();
    let resultat = match methode {
        "initialize" => {
            let version = m
                .pointer("/params/protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or("2025-06-18");
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "wikik", "version": env!("CARGO_PKG_VERSION") },
                "instructions": "Interroger wikik avant d'ouvrir des fichiers source : carte en début de session, symbole puis impact avant de modifier du code.",
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": OUTILS.iter().map(schema).collect::<Vec<_>>() }),
        "tools/call" => {
            let nom = m
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let args = m
                .pointer("/params/arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            match appeler(index, nom, &args) {
                Ok(texte) => {
                    json!({ "content": [{ "type": "text", "text": texte }], "isError": false })
                }
                Err(e) => {
                    json!({ "content": [{ "type": "text", "text": format!("{e:#}") }], "isError": true })
                }
            }
        }
        _ if id.is_none() => return None,
        _ => {
            return Some(erreur(
                id.unwrap_or(Value::Null),
                -32601,
                &format!("méthode inconnue : {methode}"),
            ));
        }
    };
    id.map(|id| json!({ "jsonrpc": "2.0", "id": id, "result": resultat }))
}

fn schema(o: &Outil) -> Value {
    let mut proprietes = serde_json::Map::new();
    let mut requis = vec![];
    for (nom, description, obligatoire) in o.parametres {
        let genre = match *nom {
            "code" => "boolean",
            "profondeur" => "integer",
            _ => "string",
        };
        proprietes.insert(
            (*nom).into(),
            json!({ "type": genre, "description": description }),
        );
        if *obligatoire {
            requis.push(*nom);
        }
    }
    json!({
        "name": o.nom,
        "description": o.description,
        "inputSchema": { "type": "object", "properties": proprietes, "required": requis },
    })
}

fn texte(args: &Value, cle: &str) -> Result<String> {
    args.get(cle)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("paramètre `{cle}` manquant"))
}

/// Exécute un outil sur l'index à jour et renvoie la sortie texte de la CLI.
/// @relu 2026-10-18 8cd7d812
fn appeler(index: &mut Index, nom: &str, args: &Value) -> Result<String> {
    index.mettre_a_jour()?;
    let i = &*index;
    let sortie = match nom {
        "carte" => query::carte(i, false)?,
        "fichier" => query::fichier(i, &texte(args, "chemin")?)?,
        "symbole" => query::symbole(
            i,
            &texte(args, "id")?,
            args.get("code").and_then(Value::as_bool).unwrap_or(false),
        )?,
        "impact" => {
            let profondeur = args.get("profondeur").and_then(Value::as_u64).unwrap_or(3) as usize;
            query::impact(i, &texte(args, "id")?, profondeur)?
        }
        "perimes" => query::perimes(i)?,
        "cherche" => query::cherche(i, &texte(args, "texte")?)?,
        "adr" => query::adr(i, &texte(args, "id")?)?,
        "exigence" => query::exigence(i, &texte(args, "id")?)?,
        "ref" => query::reference(i, &texte(args, "reference")?)?,
        "check" => return Ok(check::formater_texte(&check::verifier(i)?)),
        "atelier" => query::atelier(i)?,
        "document" => query::document_complet(i, &texte(args, "id")?)?,
        "atelier_rappel" => {
            let (id, url) =
                wikik_core::atelier::rappel(i, &texte(args, "sujet")?, &texte(args, "question")?)?;
            return Ok(format!(
                "Clarification {id} ouverte. L'utilisateur la traite dans l'atelier : {url}\nSa conclusion sera lisible avec l'outil document ({id})."
            ));
        }
        "pages" => query::pages(i)?,
        "reception" => query::reception(i)?,
        "reception_traiter" => {
            let id = texte(args, "id")?;
            let statut = texte(args, "statut")?;
            let commit = args
                .get("commit")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            wikik_core::reception::remarques::traiter(
                i,
                &id,
                &statut,
                &texte(args, "resolution")?,
                commit.as_deref(),
            )?;
            return Ok(format!("{id} : {statut}."));
        }
        _ => anyhow::bail!("outil inconnu : {nom}"),
    };
    Ok(sortie.texte)
}
