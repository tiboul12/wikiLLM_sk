//! Données des pages du wiki : structures de l'API du serveur (`wikik`), construites depuis l'index.
//!
//! Les structures sont exportées en TypeScript (`ts-rs`, dossier `front/src/types/`) pour que le
//! front et le cœur ne se désynchronisent pas (ADR-0004). Aucune entrée/sortie réseau ici.
//!
//! @req EF-041, EF-042, EF-043, EF-044
//! @adr 4

use crate::check;
use crate::index::Index;
use crate::lang::rust::resume;
use crate::projet::normaliser_adr;
use anyhow::{Result, bail};
use rusqlite::{OptionalExtension, params};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use ts_rs::TS;

/// Accueil du wiki : arbre des fichiers, liste des documents (zones Projet, atelier, réception), statistiques.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Projet {
    pub nom: String,
    pub fichiers: Vec<FichierResume>,
    pub documents: Vec<DocumentResume>,
    pub lignes: u32,
    pub symboles: u32,
    pub documentes: u32,
}

/// Fichier de code dans l'arbre de la zone Produit et le tableau fichier par fichier (EF-142).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct FichierResume {
    pub chemin: String,
    pub role: Option<String>,
    pub lignes: u32,
    pub symboles: u32,
    pub documentes: u32,
}

/// Document de la zone Projet ou de l'atelier dans la barre latérale.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DocumentResume {
    pub id: String,
    pub chemin: String,
    pub genre: String,
    pub titre: String,
    pub statut: Option<String>,
    /// `projet` ou `atelier` (dossier où vit le document).
    pub zone: String,
}

/// Lien vers un symbole ou un document, prêt à afficher.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Lien {
    /// Identifiant de symbole ou de document.
    pub id: String,
    pub libelle: String,
    /// `appel`, `rend`, `adr`, `req`, `topic`, `lien`.
    pub genre: String,
    pub confiance: String,
    /// Fichier du symbole ; `None` pour un document.
    pub fichier: Option<String>,
    /// Faux si la cible n'existe pas (balise ou lien cassé).
    pub existe: bool,
}

/// Balise d'un bloc, avec sa cible quand c'est un document.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct BaliseVue {
    pub nom: String,
    pub valeur: String,
    pub cibles: Vec<Lien>,
}

/// Cellule de la page fichier : un symbole, sa doc et son code.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Cellule {
    pub id: String,
    pub nom_qualifie: String,
    pub genre: String,
    pub visibilite: String,
    pub ligne_debut: u32,
    pub ligne_fin: u32,
    pub signature: String,
    /// Doc sans les lignes de balises (affichées à part).
    pub doc: Option<String>,
    /// `à jour`, `périmé` ou `sans doc`.
    pub statut: String,
    pub est_test: bool,
    pub code: String,
    pub balises: Vec<BaliseVue>,
    pub appelants: Vec<Lien>,
    pub appeles: Vec<Lien>,
}

/// Page d'un fichier de code : en-tête, balises d'en-tête et une cellule par symbole.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PageFichier {
    pub chemin: String,
    /// `rust` ou `typescript` : sert à la coloration du code.
    pub langage: String,
    /// En-tête `//!` sans ses lignes de balises.
    pub entete: Option<String>,
    pub balises_entete: Vec<BaliseVue>,
    pub lignes: u32,
    pub lignes_macros: u32,
    pub cellules: Vec<Cellule>,
}

/// Lien `[[...]]` d'un document et sa résolution.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct LienDocument {
    pub brute: String,
    pub cible: Option<Lien>,
}

/// Page d'un document de la zone Projet.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PageDocument {
    pub id: String,
    pub chemin: String,
    pub genre: String,
    pub titre: String,
    pub statut: Option<String>,
    pub priorite: Option<String>,
    pub version: Option<String>,
    #[ts(type = "Record<string, unknown>")]
    pub frontmatter: serde_json::Value,
    pub corps: String,
    pub liens: Vec<LienDocument>,
    /// Symboles qui citent ce document (`@adr`, `@req`).
    pub cite_par: Vec<Lien>,
    /// Documents qui pointent vers celui-ci.
    pub references: Vec<Lien>,
}

/// Nœud du graphe, avec sa catégorie et son titre de survol (EF-141).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Noeud {
    pub id: String,
    pub libelle: String,
    /// `symbole`, `fichier`, `document`, `page`, `topic`, `absent`.
    pub genre: String,
    /// Catégorie affichée (une couleur, une puce, EF-141) : `code`, `exigences`, `decisions`, `projet`,
    /// `atelier`, `pages`, `reception`, `absent`.
    pub categorie: String,
    /// Titre montré au survol (titre du document, route de la page, chemin du fichier).
    pub titre: String,
    pub fichier: Option<String>,
    pub centre: bool,
}

/// Arête du graphe.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct AreteVue {
    pub source: String,
    pub cible: String,
    pub genre: String,
    pub confiance: String,
}

/// Graphe local d'un élément, ou graphe des documents.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Graphe {
    pub noeuds: Vec<Noeud>,
    pub aretes: Vec<AreteVue>,
}

/// Résultat de recherche.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Resultat {
    pub cle: String,
    /// `symbole` ou `document`.
    pub genre: String,
    pub titre: String,
    pub extrait: String,
    pub fichier: Option<String>,
}

fn u(v: i64) -> u32 {
    u32::try_from(v).unwrap_or(0)
}

/// Accueil : fichiers avec leur rôle, documents avec leur zone (Projet, atelier, réception), statistiques.
/// @relu 2026-10-19 48f30d62
pub fn projet(index: &Index) -> Result<Projet> {
    let c = &index.conn;
    let mut st = c.prepare(
        "SELECT f.chemin, f.entete, f.lignes,
                (SELECT COUNT(*) FROM symboles s WHERE s.fichier = f.chemin AND s.est_test = 0 AND s.genre <> 'champ'),
                (SELECT COUNT(*) FROM symboles s WHERE s.fichier = f.chemin AND s.est_test = 0 AND s.genre <> 'champ' AND s.doc IS NOT NULL)
         FROM fichiers f ORDER BY f.chemin",
    )?;
    let mut lignes = 0;
    let fichiers: Vec<FichierResume> = st
        .query_map([], |r| {
            let entete: Option<String> = r.get(1)?;
            Ok((
                FichierResume {
                    chemin: r.get(0)?,
                    role: entete
                        .and_then(|e| resume(&e.lines().map(str::to_string).collect::<Vec<_>>())),
                    lignes: u(r.get(2)?),
                    symboles: u(r.get(3)?),
                    documentes: u(r.get(4)?),
                },
                r.get::<_, i64>(2)?,
            ))
        })?
        .map(|x| {
            x.map(|(f, l)| {
                lignes += l;
                f
            })
        })
        .collect::<rusqlite::Result<_>>()?;
    let dossier_atelier = format!("{}/", index.config.atelier.dossier.trim_end_matches('/'));
    let dossier_reception = format!("{}/", index.config.reception.dossier.trim_end_matches('/'));
    let mut st =
        c.prepare("SELECT id, chemin, genre, titre, statut FROM documents ORDER BY chemin")?;
    let documents = st
        .query_map([], |r| {
            let chemin: String = r.get(1)?;
            let zone = if chemin.starts_with(&dossier_atelier) {
                "atelier"
            } else if chemin.starts_with(&dossier_reception) {
                "reception"
            } else {
                "projet"
            };
            Ok(DocumentResume {
                id: r.get(0)?,
                zone: zone.into(),
                chemin,
                genre: r.get(2)?,
                titre: r.get(3)?,
                statut: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    let symboles = fichiers.iter().map(|f| f.symboles).sum();
    let documentes = fichiers.iter().map(|f| f.documentes).sum();
    let nom = index
        .racine
        .file_name()
        .map_or_else(|| "projet".into(), |n| n.to_string_lossy().into_owned());
    Ok(Projet {
        nom,
        fichiers,
        documents,
        lignes: u(lignes),
        symboles,
        documentes,
    })
}

/// Fabrique des liens à partir d'identifiants (symboles ou documents) et classe les nœuds du graphe.
struct Resolveur {
    symboles: HashMap<String, (String, String)>,
    documents: HashMap<String, String>,
    fichiers: BTreeSet<String>,
    /// Catégorie de graphe de chaque document (EF-141).
    categories: HashMap<String, &'static str>,
}

impl Resolveur {
    fn charger(index: &Index) -> Result<Resolveur> {
        let c = &index.conn;
        let mut st = c.prepare("SELECT id, fichier, nom_qualifie FROM symboles")?;
        let symboles = st
            .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))?
            .collect::<rusqlite::Result<_>>()?;
        let mut st = c.prepare("SELECT id, titre FROM documents")?;
        let documents = st
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let mut st = c.prepare("SELECT chemin FROM fichiers")?;
        let fichiers = st
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let atelier = format!("{}/", index.config.atelier.dossier.trim_end_matches('/'));
        let reception = format!("{}/", index.config.reception.dossier.trim_end_matches('/'));
        let mut st = c.prepare("SELECT id, chemin, genre FROM documents")?;
        let categories = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .map(|(id, chemin, genre)| {
                let cat = if chemin.starts_with(&reception) {
                    "reception"
                } else if chemin.starts_with(&atelier) {
                    "atelier"
                } else {
                    match genre.as_str() {
                        "besoin" | "exigence" | "exigence-nf" => "exigences",
                        "adr" => "decisions",
                        _ => "projet",
                    }
                };
                (id, cat)
            })
            .collect();
        Ok(Resolveur {
            symboles,
            documents,
            fichiers,
            categories,
        })
    }

    /// Nœud d'un identifiant : symbole, fichier, page (`page:<clé>`), document, sujet ou absent, avec sa
    /// catégorie et son titre de survol.
    fn noeud(&self, id: String, centre: bool, pages: &HashMap<String, (String, String)>) -> Noeud {
        if let Some((nom, route)) = pages.get(&id) {
            return Noeud {
                libelle: nom.clone(),
                titre: route.clone(),
                genre: "page".into(),
                categorie: "pages".into(),
                fichier: None,
                centre,
                id,
            };
        }
        let l = self.lien(&id, "", "");
        let (genre, categorie) = if self.symboles.contains_key(&id) {
            ("symbole", "code")
        } else if l.fichier.is_some() {
            ("fichier", "code")
        } else if id.starts_with("topic:") {
            ("topic", "projet")
        } else if l.existe {
            (
                "document",
                self.categories.get(&id).copied().unwrap_or("projet"),
            )
        } else {
            ("absent", "absent")
        };
        let (libelle, titre) = match genre {
            "document" => (
                id.clone(),
                self.documents.get(&id).cloned().unwrap_or_default(),
            ),
            "fichier" => (id.rsplit('/').next().unwrap_or(&id).to_string(), id.clone()),
            _ => (
                l.libelle
                    .rsplit("::")
                    .next()
                    .unwrap_or(&l.libelle)
                    .to_string(),
                l.libelle.clone(),
            ),
        };
        Noeud {
            centre,
            id,
            libelle,
            titre,
            genre: genre.into(),
            categorie: categorie.into(),
            fichier: l.fichier,
        }
    }

    fn lien(&self, id: &str, genre: &str, confiance: &str) -> Lien {
        if let Some((fichier, nq)) = self.symboles.get(id) {
            return Lien {
                id: id.into(),
                libelle: nq.clone(),
                genre: genre.into(),
                confiance: confiance.into(),
                fichier: Some(fichier.clone()),
                existe: true,
            };
        }
        if self.fichiers.contains(id) {
            // Balise d'en-tête : la source est le fichier entier.
            return Lien {
                id: id.into(),
                libelle: id.into(),
                genre: genre.into(),
                confiance: confiance.into(),
                fichier: Some(id.into()),
                existe: true,
            };
        }
        let (libelle, existe) = match self.documents.get(id) {
            Some(t) => (format!("{id} · {t}"), true),
            None => (
                id.trim_start_matches("topic:").to_string(),
                id.starts_with("topic:"),
            ),
        };
        Lien {
            id: id.into(),
            libelle,
            genre: genre.into(),
            confiance: confiance.into(),
            fichier: None,
            existe,
        }
    }
}

fn aretes(
    index: &Index,
    id: &str,
    entrantes: bool,
    genres: &[&str],
) -> Result<Vec<(String, String, String)>> {
    let (moi, autre) = if entrantes {
        ("cible", "source")
    } else {
        ("source", "cible")
    };
    let liste = genres
        .iter()
        .map(|g| format!("'{g}'"))
        .collect::<Vec<_>>()
        .join(",");
    let mut st = index.conn.prepare(&format!(
        "SELECT {autre}, genre, confiance FROM aretes WHERE {moi} = ?1 AND genre IN ({liste}) ORDER BY genre, {autre}"
    ))?;
    let v = st
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

fn balise_vue(res: &Resolveur, nom: String, valeur: String) -> BaliseVue {
    let cibles = match nom.as_str() {
        "adr" | "req" | "topic" => valeur
            .split([',', ' '])
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| {
                let cible = match nom.as_str() {
                    "adr" => normaliser_adr(v).unwrap_or_else(|| v.to_string()),
                    "req" => v.to_ascii_uppercase(),
                    _ => format!("topic:{}", v.to_lowercase()),
                };
                res.lien(&cible, &nom, "certain")
            })
            .collect(),
        _ => vec![],
    };
    BaliseVue {
        nom,
        valeur,
        cibles,
    }
}

/// Texte d'un bloc sans ses lignes de balises (affichées à part, en pastilles).
fn sans_balises(texte: &str) -> String {
    texte
        .lines()
        .filter(|l| !l.trim_start().starts_with('@'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// Colonnes d'un symbole lues pour sa cellule : id, nom qualifié, genre, visibilité, lignes, signature, doc, test.
type LigneSymbole = (
    String,
    String,
    String,
    String,
    i64,
    i64,
    String,
    Option<String>,
    bool,
);

/// Page d'un fichier : une cellule par symbole avec doc, statut, balises, voisins et code.
/// @relu 2026-10-17 59cbf684
pub fn fichier(index: &Index, chemin: &str) -> Result<PageFichier> {
    let c = &index.conn;
    let Some((entete, lignes, macros, langage)): Option<(Option<String>, i64, i64, String)> = c
        .query_row(
            "SELECT entete, lignes, lignes_macros, langage FROM fichiers WHERE chemin = ?1",
            [chemin],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
    else {
        bail!("fichier inconnu de l'index : {chemin}");
    };
    // Le chemin vient de l'index, jamais directement du client (SPEC-09 R6).
    let source = std::fs::read_to_string(index.racine.join(chemin)).unwrap_or_default();
    let texte: Vec<&str> = source.lines().collect();
    let statuts = check::statuts(index, &[chemin.to_string()])?;
    let res = Resolveur::charger(index)?;
    let mut st = c.prepare(
        "SELECT id, nom_qualifie, genre, visibilite, ligne_debut, ligne_fin, signature, doc, est_test
         FROM symboles WHERE fichier = ?1 ORDER BY ligne_debut, id",
    )?;
    let lignes_sym: Vec<LigneSymbole> = st
        .query_map([chemin], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut cellules = vec![];
    for (id, nq, genre, vis, debut, fin, signature, doc, est_test) in lignes_sym {
        let mut st =
            c.prepare("SELECT nom, valeur FROM balises WHERE symbole = ?1 ORDER BY ligne")?;
        let balises_brutes: Vec<(String, String)> = st
            .query_map([&id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let balises = balises_brutes
            .into_iter()
            .map(|(nom, valeur)| balise_vue(&res, nom, valeur))
            .collect();
        let doc_sans_balises = doc.map(|d| sans_balises(&d));
        let (d, f) = (debut.max(1) as usize, fin.max(debut) as usize);
        let code = texte
            .get(d - 1..f.min(texte.len()))
            .map(|l| l.join("\n"))
            .unwrap_or_default();
        let appelants = aretes(index, &id, true, &["appel", "rend"])?
            .into_iter()
            .map(|(x, g, conf)| res.lien(&x, &g, &conf))
            .collect();
        let appeles = aretes(index, &id, false, &["appel", "rend"])?
            .into_iter()
            .map(|(x, g, conf)| res.lien(&x, &g, &conf))
            .collect();
        cellules.push(Cellule {
            statut: statuts.get(&id).cloned().unwrap_or_else(|| "à jour".into()),
            id,
            nom_qualifie: nq,
            genre,
            visibilite: vis,
            ligne_debut: u(debut),
            ligne_fin: u(fin),
            signature,
            doc: doc_sans_balises.filter(|d| !d.is_empty()),
            est_test,
            code,
            balises,
            appelants,
            appeles,
        });
    }
    let mut st =
        c.prepare("SELECT nom, valeur FROM balises_fichier WHERE fichier = ?1 ORDER BY ligne")?;
    let balises_entete = st
        .query_map([chemin], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .map(|(n, v)| balise_vue(&res, n, v))
        .collect();
    Ok(PageFichier {
        chemin: chemin.into(),
        langage,
        entete: entete.map(|e| sans_balises(&e)).filter(|e| !e.is_empty()),
        balises_entete,
        lignes: u(lignes),
        lignes_macros: u(macros),
        cellules,
    })
}

/// Page d'un document : frontmatter, corps, liens résolus, symboles et documents qui le citent.
pub fn document(index: &Index, id: &str) -> Result<PageDocument> {
    let c = &index.conn;
    type Ligne = (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
    );
    let Some((chemin, genre, titre, statut, priorite, version, fm, corps)): Option<Ligne> = c
        .query_row(
            "SELECT chemin, genre, titre, statut, priorite, version, frontmatter, corps FROM documents WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
        )
        .optional()?
    else {
        // Identifiant tapé avec une autre casse (`plan-v1` pour `PLAN-v1`).
        let proche: Option<String> = c
            .query_row("SELECT id FROM documents WHERE lower(id) = lower(?1) LIMIT 1", [id], |r| r.get(0))
            .optional()?;
        match proche {
            Some(p) if p != id => return document(index, &p),
            _ => bail!("document inconnu : {id}"),
        }
    };
    let res = Resolveur::charger(index)?;
    let mut st = c.prepare(
        "SELECT cible_brute, cible FROM liens_doc WHERE source = ?1 ORDER BY ligne, cible_brute",
    )?;
    let liens_bruts: Vec<(String, Option<String>)> = st
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut vus = BTreeSet::new();
    let liens = liens_bruts
        .into_iter()
        .filter(|(b, _)| vus.insert(b.clone()))
        .map(|(brute, cible)| LienDocument {
            cible: cible.map(|c| res.lien(&c, "lien", "certain")),
            brute,
        })
        .collect();
    let cite_par = aretes(index, id, true, &["adr", "req"])?
        .into_iter()
        .map(|(x, g, conf)| res.lien(&x, &g, &conf))
        .collect();
    let references = aretes(index, id, true, &["lien"])?
        .into_iter()
        .map(|(x, g, conf)| res.lien(&x, &g, &conf))
        .collect();
    Ok(PageDocument {
        id: id.into(),
        chemin,
        genre,
        titre,
        statut,
        priorite,
        version,
        frontmatter: serde_json::from_str(&fm).unwrap_or_default(),
        corps,
        liens,
        cite_par,
        references,
    })
}

/// Graphe local autour de `centre` (symbole ou document), ou graphe global : documents, code ramené au
/// fichier, pages de l'application et remarques (EF-141).
///
/// @req EF-043, EF-141
pub fn graphe(index: &Index, centre: Option<&str>, profondeur: usize) -> Result<Graphe> {
    let res = Resolveur::charger(index)?;
    let mut ar: BTreeMap<(String, String, String), String> = BTreeMap::new();
    let mut pages: HashMap<String, (String, String)> = HashMap::new();
    match centre {
        Some(c0) => {
            let mut frontiere = vec![c0.to_string()];
            let mut vus: BTreeSet<String> = BTreeSet::from([c0.to_string()]);
            for _ in 0..profondeur.clamp(1, 3) {
                let mut suivante = vec![];
                for n in &frontiere {
                    let mut st = index.conn.prepare(
                        "SELECT source, cible, genre, confiance FROM aretes WHERE source = ?1 OR cible = ?1",
                    )?;
                    let v: Vec<(String, String, String, String)> = st
                        .query_map(params![n], |r| {
                            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                        })?
                        .collect::<rusqlite::Result<_>>()?;
                    for (s, cible, g, conf) in v {
                        for x in [&s, &cible] {
                            if vus.insert(x.clone()) {
                                suivante.push(x.clone());
                            }
                        }
                        ar.insert((s, cible, g), conf);
                    }
                }
                frontiere = suivante;
                if ar.len() > 300 {
                    break;
                }
            }
        }
        None => {
            // Graphe global (EF-141) : documents et leurs liens, code ramené au fichier, pages de
            // l'application et remarques de réception.
            let fichier_de = |id: &str| -> String {
                res.symboles
                    .get(id)
                    .map_or_else(|| id.to_string(), |(f, _)| f.clone())
            };
            let mut st = index.conn.prepare(
                "SELECT source, cible, genre, confiance FROM aretes WHERE genre IN ('lien', 'req', 'adr')",
            )?;
            let v: Vec<(String, String, String, String)> = st
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
                .collect::<rusqlite::Result<_>>()?;
            for (s, c, g, conf) in v {
                let (s, c) = (fichier_de(&s), fichier_de(&c));
                if s != c {
                    ar.insert((s, c, g), conf);
                }
            }
            for id in res.documents.keys() {
                ar.entry((id.clone(), id.clone(), "_".into())).or_default();
            }
            let carte = crate::reception::carte::fusionner(
                &crate::reception::carte::lire(index),
                crate::reception::carte::detecter(index).unwrap_or_default(),
            );
            for p in carte.iter().filter(|p| !p.masque) {
                let id = format!("page:{}", p.cle);
                pages.insert(
                    id.clone(),
                    (p.nom.clone(), p.route.clone().unwrap_or_default()),
                );
                ar.entry((id.clone(), id.clone(), "_".into())).or_default();
                if let Some(f) = p
                    .symboles
                    .first()
                    .map(|s| fichier_de(s))
                    .or(p.fichier.clone())
                {
                    ar.insert((id.clone(), f, "rend".into()), "certain".into());
                }
                for e in &p.exigences {
                    ar.insert((id.clone(), e.clone(), "req".into()), "certain".into());
                }
            }
            for r in crate::reception::remarques::lister(index).unwrap_or_default() {
                // Une version remplacée par une modification n'est plus une remarque à part.
                if r.statut == "remplacee" {
                    ar.retain(|k, _| k.0 != r.id && k.1 != r.id);
                    continue;
                }
                ar.insert(
                    (r.id.clone(), format!("page:{}", r.page), "remarque".into()),
                    "certain".into(),
                );
            }
        }
    }
    let mut ids: BTreeSet<String> = BTreeSet::new();
    let aretes: Vec<AreteVue> = ar
        .into_iter()
        .filter_map(|((s, c, g), conf)| {
            ids.insert(s.clone());
            ids.insert(c.clone());
            (g != "_").then_some(AreteVue {
                source: s,
                cible: c,
                genre: g,
                confiance: conf,
            })
        })
        .collect();
    let noeuds = ids
        .into_iter()
        .map(|id| {
            let c = Some(id.as_str()) == centre;
            res.noeud(id, c, &pages)
        })
        .collect();
    Ok(Graphe { noeuds, aretes })
}

/// Recherche plein texte, mêmes règles que `wikik query cherche`.
pub fn cherche(index: &Index, texte: &str) -> Result<Vec<Resultat>> {
    let mots: Vec<String> = texte
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|m| !m.is_empty())
        .map(|m| format!("\"{m}\"*"))
        .collect();
    if mots.is_empty() {
        return Ok(vec![]);
    }
    let mut st = index.conn.prepare(
        "SELECT r.cle, r.genre, r.titre, snippet(recherche, 3, '', '', '…', 16), s.fichier
         FROM recherche r LEFT JOIN symboles s ON s.id = r.cle
         WHERE recherche MATCH ?1 ORDER BY rank LIMIT 30",
    )?;
    let v = st
        .query_map([mots.join(" ")], |r| {
            Ok(Resultat {
                cle: r.get(0)?,
                genre: r.get(1)?,
                titre: r.get(2)?,
                extrait: r.get(3)?,
                fichier: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Demande au LLM de la recherche assistée (EF-152) : la question, les résultats plein texte comme
/// point de départ (un modèle sans MCP n'a que ceux-là), et le schéma de la réponse : des éléments
/// `{ cible, raison }` où `cible` est un identifiant de document, un chemin de fichier ou `chemin::Symbole`,
/// que le front transforme en lien.
/// @req EF-152
pub fn demande_recherche(index: &Index, question: &str) -> Result<(String, serde_json::Value)> {
    let question = question.trim();
    if question.is_empty() {
        bail!("question vide");
    }
    let indices: Vec<String> = cherche(index, question)?
        .into_iter()
        .take(15)
        .map(|r| format!("- {} ({}) : {}", r.cle, r.genre, r.titre))
        .collect();
    let consigne = format!(
        "Tu aides l'utilisateur à trouver ce qui répond à sa question dans son projet (code, documents, décisions).\n\
         Lis le projet avec les outils wikik si tu les as (carte, cherche, symbole, document) ; tu n'écris rien.\n\
         Méthode de lecture du wiki : {}\n\n\
         Question : {question}\n\n\
         Résultats de la recherche plein texte, comme point de départ :\n{}\n\n\
         Réponds par les éléments du projet qui répondent le mieux, du plus utile au moins utile (8 au plus). \
         Pour chacun : `cible`, l'identifiant exact (ex. `EF-140`, `crates/x/src/a.rs`, `crates/x/src/a.rs::Type::f`), \
         et `raison`, une ou deux phrases qui disent ce qu'on y trouve. N'invente aucune cible ; `elements: []` si rien ne répond.",
        index.config.ia.consigne_lecture,
        if indices.is_empty() {
            "(aucun)".to_string()
        } else {
            indices.join("\n")
        }
    );
    let schema = serde_json::json!({
        "type": "object", "required": ["elements"],
        "properties": { "elements": { "type": "array", "items": {
            "type": "object", "required": ["cible", "raison"],
            "properties": { "cible": {"type": "string"}, "raison": {"type": "string"} }
        }}}
    });
    Ok((consigne, schema))
}

/// Entrée de l'index de recherche du site statique (SPEC-16 R1).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct EntreeRecherche {
    pub cle: String,
    /// `symbole` ou `document`.
    pub genre: String,
    pub titre: String,
    /// Doc du symbole ou corps du document, 2 000 caractères au plus.
    pub texte: String,
    pub fichier: Option<String>,
}

/// Tous les nœuds et toutes les arêtes : le site statique en tire les graphes locaux.
///
/// @req EF-080
/// @relu 2026-10-19 163c1ae1
pub fn graphe_complet(index: &Index) -> Result<Graphe> {
    let res = Resolveur::charger(index)?;
    let mut st = index.conn.prepare(
        "SELECT source, cible, genre, confiance FROM aretes ORDER BY source, cible, genre",
    )?;
    let aretes: Vec<AreteVue> = st
        .query_map([], |r| {
            Ok(AreteVue {
                source: r.get(0)?,
                cible: r.get(1)?,
                genre: r.get(2)?,
                confiance: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut ids: BTreeSet<String> = res.symboles.keys().cloned().collect();
    ids.extend(res.documents.keys().cloned());
    for a in &aretes {
        ids.insert(a.source.clone());
        ids.insert(a.cible.clone());
    }
    let pages = HashMap::new();
    let noeuds = ids
        .into_iter()
        .map(|id| res.noeud(id, false, &pages))
        .collect();
    Ok(Graphe { noeuds, aretes })
}

/// Index de recherche du site statique : symboles documentés et documents.
///
/// @req EF-080
pub fn donnees_recherche(index: &Index) -> Result<Vec<EntreeRecherche>> {
    let tronquer = |t: String| {
        if t.chars().count() > 2000 {
            t.chars().take(2000).collect()
        } else {
            t
        }
    };
    let mut v = vec![];
    let mut st = index.conn.prepare(
        "SELECT id, nom_qualifie, coalesce(doc, '') || ' ' || signature, fichier FROM symboles WHERE est_test = 0 ORDER BY id",
    )?;
    for r in st.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get::<_, String>(2)?, r.get(3)?))
    })? {
        let (cle, titre, texte, fichier) = r?;
        v.push(EntreeRecherche {
            cle,
            genre: "symbole".into(),
            titre,
            texte: tronquer(texte),
            fichier: Some(fichier),
        });
    }
    let mut st = index
        .conn
        .prepare("SELECT id, titre, corps FROM documents ORDER BY id")?;
    for r in st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, String>(2)?)))? {
        let (cle, titre, texte) = r?;
        v.push(EntreeRecherche {
            cle,
            genre: "document".into(),
            titre,
            texte: tronquer(texte),
            fichier: None,
        });
    }
    Ok(v)
}
