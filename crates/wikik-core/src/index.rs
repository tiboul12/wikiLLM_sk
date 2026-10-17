//! Index SQLite du projet : création, mise à jour incrémentale et dump JSON déterministe.
//!
//! Une seule voie d'entrée : [`Index::ouvrir`] puis [`Index::mettre_a_jour`], que toute requête
//! appelle avant de répondre (EF-029). L'index est un artefact reconstructible (ADR-0003).
//!
//! @req EF-001, EF-002, EF-003, EF-029, ENF-002
//! @adr 3

use crate::config::Config;
use crate::hash::empreinte;
use crate::lang::{self, Langage};
use crate::modele::ExtraitFichier;
use crate::projet::{self, Document};
use crate::{SCHEMA, resolution};
use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Dossier de travail de wikik dans un projet (ignoré par git).
pub const DOSSIER: &str = ".wikik";

const SCHEMA_SQL: &str = r#"
CREATE TABLE meta (cle TEXT PRIMARY KEY, valeur TEXT NOT NULL);
CREATE TABLE etat (chemin TEXT PRIMARY KEY, taille INTEGER NOT NULL, mtime INTEGER NOT NULL, hash TEXT NOT NULL);
CREATE TABLE fichiers (chemin TEXT PRIMARY KEY, langage TEXT NOT NULL, hash_contenu TEXT NOT NULL, entete TEXT,
  lignes INTEGER NOT NULL, lignes_macros INTEGER NOT NULL, erreurs_parse INTEGER NOT NULL);
CREATE TABLE symboles (id TEXT PRIMARY KEY, fichier TEXT NOT NULL REFERENCES fichiers(chemin) ON DELETE CASCADE,
  nom TEXT NOT NULL, nom_qualifie TEXT NOT NULL, genre TEXT NOT NULL, visibilite TEXT NOT NULL,
  ligne_debut INTEGER NOT NULL, ligne_fin INTEGER NOT NULL, doc_ligne_debut INTEGER, doc_ligne_fin INTEGER,
  signature TEXT NOT NULL, resume TEXT, doc TEXT, hash_corps TEXT NOT NULL, hash_signature TEXT NOT NULL,
  hash_doc TEXT, est_test INTEGER NOT NULL, type_parent TEXT);
CREATE INDEX symboles_fichier ON symboles(fichier);
CREATE INDEX symboles_nom ON symboles(nom);
CREATE TABLE balises (symbole TEXT NOT NULL REFERENCES symboles(id) ON DELETE CASCADE,
  nom TEXT NOT NULL, valeur TEXT NOT NULL, ligne INTEGER NOT NULL);
CREATE TABLE appels_bruts (symbole TEXT NOT NULL REFERENCES symboles(id) ON DELETE CASCADE,
  forme TEXT NOT NULL, cible TEXT NOT NULL, qualificatif TEXT, ligne INTEGER NOT NULL);
CREATE TABLE imports (fichier TEXT NOT NULL REFERENCES fichiers(chemin) ON DELETE CASCADE,
  alias TEXT NOT NULL, chemin_import TEXT NOT NULL);
CREATE TABLE aretes (source TEXT NOT NULL, cible TEXT NOT NULL, genre TEXT NOT NULL, confiance TEXT NOT NULL,
  ligne INTEGER, PRIMARY KEY (source, cible, genre));
CREATE INDEX aretes_cible ON aretes(cible);
CREATE TABLE documents (id TEXT PRIMARY KEY, chemin TEXT NOT NULL UNIQUE, genre TEXT NOT NULL, titre TEXT NOT NULL,
  statut TEXT, version TEXT, priorite TEXT, frontmatter TEXT NOT NULL, frontmatter_invalide TEXT,
  corps TEXT NOT NULL, hash_contenu TEXT NOT NULL);
CREATE TABLE liens_doc (source TEXT NOT NULL, cible_brute TEXT NOT NULL, cible TEXT, ligne INTEGER NOT NULL);
CREATE VIRTUAL TABLE recherche USING fts5(cle UNINDEXED, genre UNINDEXED, titre, texte,
  tokenize = 'unicode61 remove_diacritics 2');
"#;

/// Bilan d'une mise à jour, affiché par la CLI.
#[derive(Debug, Clone, Default)]
pub struct Rapport {
    pub fichiers: usize,
    pub analyses: usize,
    pub supprimes: usize,
    pub symboles: usize,
    pub documents: usize,
    pub duree_ms: u128,
    pub reconstruit: bool,
}

/// Index ouvert sur un projet.
pub struct Index {
    pub racine: PathBuf,
    pub config: Config,
    pub conn: Connection,
}

/// Fichier candidat à l'indexation.
struct Candidat {
    relatif: String,
    absolu: PathBuf,
    taille: i64,
    mtime: i64,
    genre: Genre,
}

#[derive(Clone, Copy, PartialEq)]
enum Genre {
    Code(usize),
    Document,
}

impl Index {
    /// Ouvre (ou crée) `.wikik/index.sqlite` ; un schéma d'une autre version est supprimé et recréé.
    pub fn ouvrir(racine: &Path) -> Result<Index> {
        let racine = racine
            .canonicalize()
            .with_context(|| format!("racine introuvable : {}", racine.display()))?;
        let config = Config::charger(&racine)?;
        let dossier = racine.join(DOSSIER);
        std::fs::create_dir_all(&dossier)
            .with_context(|| format!("création de {}", dossier.display()))?;
        let chemin = dossier.join("index.sqlite");
        let mut conn = Connection::open(&chemin)?;
        let version: Option<String> = conn
            .query_row("SELECT valeur FROM meta WHERE cle = 'schema'", [], |r| {
                r.get(0)
            })
            .optional()
            .unwrap_or(None);
        if version.as_deref() != Some(&SCHEMA.to_string()) {
            drop(conn);
            let _ = std::fs::remove_file(&chemin);
            conn = Connection::open(&chemin)?;
            conn.execute_batch(SCHEMA_SQL)?;
            conn.execute(
                "INSERT INTO meta VALUES ('schema', ?1)",
                [SCHEMA.to_string()],
            )?;
        }
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;",
        )?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        Ok(Index {
            racine,
            config,
            conn,
        })
    }

    /// Met l'index à jour : seuls les fichiers dont la taille, la date ou le contenu ont changé sont relus.
    pub fn mettre_a_jour(&mut self) -> Result<Rapport> {
        let debut = Instant::now();
        let langages = lang::actives(&self.config.index.langages);
        let candidats = self.lister(&langages)?;
        let mut rapport = Rapport {
            fichiers: candidats.len(),
            ..Default::default()
        };

        let etat: BTreeMap<String, (i64, i64, String)> = {
            let mut st = self
                .conn
                .prepare("SELECT chemin, taille, mtime, hash FROM etat")?;
            st.query_map([], |r| {
                Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?)))
            })?
            .collect::<rusqlite::Result<_>>()?
        };

        let tx = self.conn.transaction()?;
        let presents: std::collections::BTreeSet<&str> =
            candidats.iter().map(|c| c.relatif.as_str()).collect();
        for chemin in etat.keys().filter(|k| !presents.contains(k.as_str())) {
            tx.execute("DELETE FROM fichiers WHERE chemin = ?1", [chemin])?;
            tx.execute("DELETE FROM liens_doc WHERE source IN (SELECT id FROM documents WHERE chemin = ?1)", [chemin])?;
            tx.execute("DELETE FROM documents WHERE chemin = ?1", [chemin])?;
            tx.execute("DELETE FROM etat WHERE chemin = ?1", [chemin])?;
            rapport.supprimes += 1;
        }

        let dossier_projet = self.config.projet.dossier.trim_end_matches('/').to_string();
        for c in &candidats {
            if let Some((taille, mtime, _)) = etat.get(&c.relatif)
                && *taille == c.taille
                && *mtime == c.mtime
            {
                continue;
            }
            let octets =
                std::fs::read(&c.absolu).with_context(|| format!("lecture de {}", c.relatif))?;
            let hash = empreinte(&octets);
            let inchange = etat.get(&c.relatif).is_some_and(|(_, _, h)| *h == hash);
            tx.execute(
                "INSERT OR REPLACE INTO etat VALUES (?1, ?2, ?3, ?4)",
                params![c.relatif, c.taille, c.mtime, hash],
            )?;
            if inchange {
                continue;
            }
            rapport.analyses += 1;
            match c.genre {
                Genre::Code(i) => {
                    let extrait = langages[i].extraire(&c.relatif, &octets);
                    ecrire_fichier(&tx, &c.relatif, langages[i].nom(), &hash, &extrait)?;
                }
                Genre::Document => {
                    let texte = String::from_utf8_lossy(&octets);
                    let relatif_projet = c
                        .relatif
                        .strip_prefix(&format!("{dossier_projet}/"))
                        .unwrap_or(&c.relatif);
                    let doc = projet::lire(&c.relatif, relatif_projet, &texte);
                    ecrire_document(&tx, &doc)?;
                }
            }
        }

        if rapport.analyses > 0 || rapport.supprimes > 0 {
            resolution::recalculer(&tx)?;
            reconstruire_recherche(&tx)?;
        }
        tx.commit()?;
        rapport.symboles = self
            .conn
            .query_row("SELECT COUNT(*) FROM symboles", [], |r| r.get::<_, i64>(0))?
            as usize;
        rapport.documents = self
            .conn
            .query_row("SELECT COUNT(*) FROM documents", [], |r| r.get::<_, i64>(0))?
            as usize;
        rapport.reconstruit = etat.is_empty();
        rapport.duree_ms = debut.elapsed().as_millis();
        Ok(rapport)
    }

    fn lister(&self, langages: &[Box<dyn Langage>]) -> Result<Vec<Candidat>> {
        let mut surcharges = ignore::overrides::OverrideBuilder::new(&self.racine);
        for motif in &self.config.index.exclure {
            surcharges.add(&format!("!{motif}"))?;
        }
        let surcharges = surcharges.build()?;
        let marcheur = ignore::WalkBuilder::new(&self.racine)
            .hidden(true)
            .require_git(false)
            .overrides(surcharges)
            .filter_entry(|e| {
                let n = e.file_name().to_string_lossy();
                !(e.file_type().is_some_and(|t| t.is_dir())
                    && matches!(n.as_ref(), "target" | "node_modules" | ".git" | DOSSIER))
            })
            .build();
        let dossier_projet = format!("{}/", self.config.projet.dossier.trim_end_matches('/'));
        let mut v = vec![];
        for entree in marcheur {
            let entree = entree?;
            if !entree.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let absolu = entree.path().to_path_buf();
            let Ok(rel) = absolu.strip_prefix(&self.racine) else {
                continue;
            };
            let relatif = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let ext = absolu.extension().and_then(|e| e.to_str()).unwrap_or("");
            let genre = if let Some(i) = langages.iter().position(|l| l.extensions().contains(&ext))
            {
                Genre::Code(i)
            } else if ext == "md" && relatif.starts_with(&dossier_projet) {
                Genre::Document
            } else {
                continue;
            };
            let meta = entree.metadata()?;
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_nanos() as i64);
            v.push(Candidat {
                relatif,
                absolu,
                taille: meta.len() as i64,
                mtime,
                genre,
            });
        }
        v.sort_by(|a, b| a.relatif.cmp(&b.relatif));
        Ok(v)
    }

    /// Dump JSON déterministe : trié, sans horodatage ni identifiant interne (SPEC-02).
    pub fn dump_json(&self) -> Result<Value> {
        let c = &self.conn;
        let fichiers = requete_json(
            c,
            "SELECT chemin, langage, hash_contenu, entete, lignes, lignes_macros, erreurs_parse FROM fichiers ORDER BY chemin",
            &[
                "chemin",
                "langage",
                "hash_contenu",
                "entete",
                "lignes",
                "lignes_macros",
                "erreurs_parse",
            ],
        )?;
        let mut symboles = requete_json(
            c,
            "SELECT id, fichier, nom, nom_qualifie, genre, visibilite, ligne_debut, ligne_fin, doc_ligne_debut, doc_ligne_fin,
                    signature, resume, doc, hash_corps, hash_signature, hash_doc, est_test, type_parent FROM symboles ORDER BY id",
            &[
                "id", "fichier", "nom", "nom_qualifie", "genre", "visibilite", "ligne_debut", "ligne_fin", "doc_ligne_debut",
                "doc_ligne_fin", "signature", "resume", "doc", "hash_corps", "hash_signature", "hash_doc", "est_test", "type_parent",
            ],
        )?;
        let balises = requete_json(
            c,
            "SELECT symbole, nom, valeur, ligne FROM balises ORDER BY symbole, ligne, nom, valeur",
            &["symbole", "nom", "valeur", "ligne"],
        )?;
        let mut par_symbole: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for mut b in balises {
            let s = b["symbole"].as_str().unwrap_or_default().to_string();
            b.as_object_mut().unwrap().remove("symbole");
            par_symbole.entry(s).or_default().push(b);
        }
        for s in &mut symboles {
            let id = s["id"].as_str().unwrap_or_default().to_string();
            s["balises"] = Value::Array(par_symbole.remove(&id).unwrap_or_default());
        }
        let aretes = requete_json(
            c,
            "SELECT source, cible, genre, confiance, ligne FROM aretes ORDER BY source, cible, genre",
            &["source", "cible", "genre", "confiance", "ligne"],
        )?;
        let mut documents = requete_json(
            c,
            "SELECT id, chemin, genre, titre, statut, version, priorite, frontmatter, frontmatter_invalide, hash_contenu
             FROM documents ORDER BY id",
            &["id", "chemin", "genre", "titre", "statut", "version", "priorite", "frontmatter", "frontmatter_invalide", "hash_contenu"],
        )?;
        for d in &mut documents {
            let fm = d["frontmatter"].as_str().unwrap_or("{}").to_string();
            d["frontmatter"] = serde_json::from_str(&fm).unwrap_or(Value::Null);
        }
        let liens = requete_json(
            c,
            "SELECT source, cible_brute, cible, ligne FROM liens_doc ORDER BY source, ligne, cible_brute",
            &["source", "cible_brute", "cible", "ligne"],
        )?;
        Ok(json!({
            "schema": SCHEMA,
            "fichiers": fichiers,
            "symboles": symboles,
            "aretes": aretes,
            "documents": documents,
            "liens_doc": liens,
        }))
    }
}

fn ecrire_fichier(
    tx: &rusqlite::Transaction,
    chemin: &str,
    langage: &str,
    hash: &str,
    e: &ExtraitFichier,
) -> Result<()> {
    tx.execute("DELETE FROM fichiers WHERE chemin = ?1", [chemin])?;
    tx.execute(
        "INSERT INTO fichiers VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            chemin,
            langage,
            hash,
            e.entete,
            e.lignes as i64,
            e.lignes_macros as i64,
            e.erreurs_parse as i64
        ],
    )?;
    let mut ins_s = tx.prepare_cached(
        "INSERT OR REPLACE INTO symboles VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
    )?;
    let mut ins_b = tx.prepare_cached("INSERT INTO balises VALUES (?1, ?2, ?3, ?4)")?;
    let mut ins_a = tx.prepare_cached("INSERT INTO appels_bruts VALUES (?1, ?2, ?3, ?4, ?5)")?;
    for s in &e.symboles {
        ins_s.execute(params![
            s.id,
            chemin,
            s.nom,
            s.nom_qualifie,
            s.genre,
            s.visibilite,
            s.ligne_debut as i64,
            s.ligne_fin as i64,
            s.doc_ligne_debut.map(|v| v as i64),
            s.doc_ligne_fin.map(|v| v as i64),
            s.signature,
            s.resume,
            s.doc,
            s.hash_corps,
            s.hash_signature,
            s.hash_doc,
            s.est_test as i64,
            s.type_parent,
        ])?;
        for b in &s.balises {
            ins_b.execute(params![s.id, b.nom, b.valeur, b.ligne as i64])?;
        }
        for a in &s.appels {
            ins_a.execute(params![
                s.id,
                a.forme,
                a.cible,
                a.qualificatif,
                a.ligne as i64
            ])?;
        }
    }
    let mut ins_i = tx.prepare_cached("INSERT INTO imports VALUES (?1, ?2, ?3)")?;
    for i in &e.imports {
        ins_i.execute(params![chemin, i.alias, i.chemin])?;
    }
    Ok(())
}

fn ecrire_document(tx: &rusqlite::Transaction, d: &Document) -> Result<()> {
    tx.execute(
        "DELETE FROM liens_doc WHERE source IN (SELECT id FROM documents WHERE chemin = ?1)",
        [&d.chemin],
    )?;
    tx.execute("DELETE FROM documents WHERE chemin = ?1", [&d.chemin])?;
    // Deux documents avec le même identifiant : le second garde son chemin comme identifiant,
    // `check` signalera le doublon.
    let existe: bool = tx
        .query_row("SELECT 1 FROM documents WHERE id = ?1", [&d.id], |_| {
            Ok(true)
        })
        .optional()?
        .unwrap_or(false);
    let id = if existe {
        format!("{}@{}", d.id, d.chemin)
    } else {
        d.id.clone()
    };
    tx.execute(
        "INSERT INTO documents VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            id,
            d.chemin,
            d.genre,
            d.titre,
            d.statut,
            d.version,
            d.priorite,
            serde_json::to_string(&d.frontmatter)?,
            d.frontmatter_invalide,
            d.corps,
            d.hash_contenu
        ],
    )?;
    // Liens bruts ; leur cible est calculée par la résolution globale.
    for l in &d.liens {
        tx.execute(
            "INSERT INTO liens_doc VALUES (?1, ?2, NULL, ?3)",
            params![id, l.cible, l.ligne as i64],
        )?;
    }
    Ok(())
}

fn reconstruire_recherche(tx: &rusqlite::Transaction) -> Result<()> {
    tx.execute("DELETE FROM recherche", [])?;
    tx.execute(
        "INSERT INTO recherche (cle, genre, titre, texte)
         SELECT id, 'symbole', nom_qualifie, coalesce(doc, '') || ' ' || signature FROM symboles",
        [],
    )?;
    tx.execute(
        "INSERT INTO recherche (cle, genre, titre, texte) SELECT id, 'document', titre, corps FROM documents",
        [],
    )?;
    Ok(())
}

/// Exécute une requête et renvoie chaque ligne en objet JSON (colonnes nommées par `noms`).
pub fn requete_json(c: &Connection, sql: &str, noms: &[&str]) -> Result<Vec<Value>> {
    let mut st = c.prepare(sql)?;
    let lignes = st.query_map([], |r| {
        let mut m = serde_json::Map::new();
        for (i, n) in noms.iter().enumerate() {
            let v = match r.get_ref(i)? {
                rusqlite::types::ValueRef::Null => Value::Null,
                rusqlite::types::ValueRef::Integer(x) => json!(x),
                rusqlite::types::ValueRef::Real(x) => json!(x),
                rusqlite::types::ValueRef::Text(t) => {
                    Value::String(String::from_utf8_lossy(t).into_owned())
                }
                rusqlite::types::ValueRef::Blob(_) => Value::Null,
            };
            m.insert((*n).to_string(), v);
        }
        Ok(Value::Object(m))
    })?;
    Ok(lignes.collect::<rusqlite::Result<Vec<_>>>()?)
}
