//! Carte des pages d'une application, construite à partir de son code (SPEC-22) : routes déclarées,
//! pages SvelteKit, énumération de navigation, pages HTML, composants rangés dans un dossier de pages.
//!
//! Le scan ne demande aucune configuration : il lit les fichiers que l'index parcourt et relie chaque
//! page aux symboles de l'index (composant, exigences `@req`). La carte est un fichier du dépôt ; les
//! corrections de Tibo (nom, masque, groupe) survivent aux scans suivants.
//!
//! @req EF-120, EF-121
//! @adr 10

use crate::index::Index;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;
use ts_rs::TS;

/// Une page de l'application reçue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Page {
    /// Clé stable : la route, sinon le nom du composant ou du fichier, en minuscules.
    pub cle: String,
    pub nom: String,
    #[serde(default)]
    pub route: Option<String>,
    /// Symboles de l'index qui font la page (le composant d'abord).
    #[serde(default)]
    pub symboles: Vec<String>,
    /// Exigences citées (`@req`) par ces symboles.
    #[serde(default)]
    pub exigences: Vec<String>,
    #[serde(default)]
    pub fichier: Option<String>,
    /// Détecteur : `route`, `fichier-route`, `enum`, `html`, `composant`, `llm`, `manuel`.
    pub source: String,
    #[serde(default)]
    pub groupe: Option<String>,
    #[serde(default)]
    pub masque: bool,
    /// Nom, masque ou groupe modifiés par Tibo : le scan ne les touche plus.
    #[serde(default)]
    pub corrige: bool,
    /// Plus trouvée par le dernier scan (gardée pour ses remarques).
    #[serde(default)]
    pub disparue: bool,
}

/// Page des remarques qui ne visent aucun écran en particulier.
pub const GENERAL: &str = "general";

const EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "mjs", "vue", "svelte", "html",
];
const DOSSIERS_PAGES: &[&str] = &["pages", "views", "routes", "screens", "ecrans", "vues"];
/// Balises de structure des frameworks : jamais prises pour le composant d'une page.
const STRUCTURE: &[&str] = &[
    "Show",
    "For",
    "Suspense",
    "Transition",
    "ErrorBoundary",
    "Await",
    "Outlet",
    "Router",
    "Routes",
    "Route",
    "A",
    "Html",
    "Title",
    "Meta",
    "Fragment",
    "Provider",
    "Link",
    "NavLink",
    "Redirect",
    "Navigate",
];

static OBJET_ROUTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\{\s*path\s*:\s*['"`]([^'"`]*)['"`]([^{}]*)\}"#).unwrap());
static COMPOSANT_ROUTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:component|Component|element)\s*:\s*<?\s*([A-Z]\w*)|\bm\s*\.\s*([A-Z]\w*)")
        .unwrap()
});
static BALISE_ROUTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<Route\b[^>]{0,400}").unwrap());
static CHEMIN_BALISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\bpath\s*=\s*\{?\s*(?:path!\(\s*|StaticSegment\(\s*)?["']([^"']*)["']"#).unwrap()
});
static VUE_BALISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:view|element|component|Component)\s*=\s*\{?\s*<?\s*([A-Z]\w*)").unwrap()
});
static ENUM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\benum\s+(Page|Pages|Screen|Screens|View|Views|Ecran|Ecrans|Vue|Vues)\s*\{([^}]*)\}",
    )
    .unwrap()
});
static TITRE_HTML: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<title>\s*([^<]*?)\s*</title>").unwrap());
static BALISE_MAJ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<([A-Z]\w*)").unwrap());

struct Source {
    chemin: String,
    ext: String,
    texte: String,
}

fn sources(index: &Index) -> Result<Vec<Source>> {
    let mut v = vec![];
    for e in index.marcheur()? {
        let e = e?;
        if !e.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let ext = e
            .path()
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_string();
        if !EXTENSIONS.contains(&ext.as_str()) || e.metadata().map_or(true, |m| m.len() > 1_000_000)
        {
            continue;
        }
        let Ok(rel) = e.path().strip_prefix(&index.racine) else {
            continue;
        };
        let chemin = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if chemin.ends_with(".d.ts")
            || chemin.ends_with(".min.js")
            || chemin.split('/').any(|d| matches!(d, "dist" | "build"))
        {
            continue;
        }
        let Ok(texte) = std::fs::read_to_string(e.path()) else {
            continue;
        };
        v.push(Source { chemin, ext, texte });
    }
    v.sort_by(|a, b| a.chemin.cmp(&b.chemin));
    Ok(v)
}

/// « TemplatePage » → « Template page », « mes-cv » → « Mes cv ».
fn humaniser(t: &str) -> String {
    let mut mots = String::new();
    let mut prec_min = false;
    for c in t.chars() {
        if c == '-' || c == '_' || c == '/' {
            mots.push(' ');
            prec_min = false;
            continue;
        }
        if c.is_uppercase() && prec_min {
            mots.push(' ');
        }
        prec_min = c.is_lowercase() || c.is_ascii_digit();
        mots.extend(c.to_lowercase());
    }
    let mots = mots.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut c = mots.chars();
    c.next()
        .map(|p| p.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

fn route_normale(r: &str) -> String {
    let r = r.trim().trim_matches('/');
    format!("/{r}")
}

/// Symbole de l'index qui porte ce nom (composant, classe ou fonction), hors tests.
fn symbole(index: &Index, nom: &str) -> Option<(String, String)> {
    index
        .conn
        .query_row(
            "SELECT id, fichier FROM symboles WHERE nom = ?1 AND est_test = 0
             ORDER BY CASE genre WHEN 'composant' THEN 0 WHEN 'class' THEN 1 WHEN 'fn' THEN 2 ELSE 3 END, id LIMIT 1",
            [nom],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok()
}

fn exigences(index: &Index, symboles: &[String]) -> Vec<String> {
    let mut v = BTreeSet::new();
    for s in symboles {
        if let Ok(mut st) = index
            .conn
            .prepare("SELECT cible FROM aretes WHERE source = ?1 AND genre = 'req'")
            && let Ok(l) = st.query_map([s], |r| r.get::<_, String>(0))
        {
            v.extend(l.flatten());
        }
    }
    v.into_iter().collect()
}

/// Une page trouvée par un détecteur, reliée au symbole de son composant et à ses exigences.
#[allow(clippy::too_many_arguments)]
fn page(
    index: &Index,
    cle: String,
    nom: String,
    route: Option<String>,
    composant: Option<&str>,
    fichier: Option<String>,
    source: &str,
    groupe: Option<String>,
) -> Page {
    let trouve = composant.and_then(|c| symbole(index, c));
    let symboles: Vec<String> = trouve.iter().map(|(id, _)| id.clone()).collect();
    Page {
        cle,
        nom,
        route,
        exigences: exigences(index, &symboles),
        fichier: trouve.map(|(_, f)| f).or(fichier),
        symboles,
        source: source.into(),
        groupe,
        masque: false,
        corrige: false,
        disparue: false,
    }
}

/// Applique les détecteurs (SPEC-22 R2), dans l'ordre ; une route ou un composant déjà vus ne sont
/// pas repris.
/// @relu 2026-10-18 501afbd6
pub fn detecter(index: &Index) -> Result<Vec<Page>> {
    let fichiers = sources(index)?;
    let mut pages: Vec<Page> = vec![];
    let mut vus_routes = BTreeSet::new();
    let mut vus_composants = BTreeSet::new();
    let mut ajouter = |p: Page, composant: Option<String>, pages: &mut Vec<Page>| {
        if let Some(r) = &p.route
            && !vus_routes.insert(r.clone())
        {
            return;
        }
        if let Some(c) = composant
            && !vus_composants.insert(c)
        {
            return;
        }
        if pages.iter().any(|x| x.cle == p.cle) {
            return;
        }
        pages.push(p);
    };

    // 1. Routes déclarées : objets (Angular, Vue, React Router en objets) et balises <Route> (React, Leptos).
    for f in fichiers.iter().filter(|f| f.ext != "html") {
        for m in OBJET_ROUTE.captures_iter(&f.texte) {
            let reste = &m[2];
            if reste.contains("redirectTo") || reste.contains("redirect:") {
                continue;
            }
            let Some(c) = COMPOSANT_ROUTE.captures(reste) else {
                continue;
            };
            let comp = c.get(1).or(c.get(2)).unwrap().as_str().to_string();
            let route = route_normale(&m[1]);
            let nom = if route == "/" {
                humaniser(comp.trim_end_matches("Component").trim_end_matches("Page"))
            } else {
                humaniser(route.trim_start_matches('/'))
            };
            let p = page(
                index,
                route.clone(),
                nom,
                Some(route),
                Some(&comp),
                Some(f.chemin.clone()),
                "route",
                None,
            );
            ajouter(p, Some(comp), &mut pages);
        }
        for m in BALISE_ROUTE.find_iter(&f.texte) {
            let balise = m.as_str();
            let (Some(ch), Some(vue)) =
                (CHEMIN_BALISE.captures(balise), VUE_BALISE.captures(balise))
            else {
                continue;
            };
            let comp = vue[1].to_string();
            let route = route_normale(&ch[1]);
            let nom = if route == "/" {
                humaniser(comp.trim_end_matches("Page"))
            } else {
                humaniser(route.trim_start_matches('/'))
            };
            let p = page(
                index,
                route.clone(),
                nom,
                Some(route),
                Some(&comp),
                Some(f.chemin.clone()),
                "route",
                None,
            );
            ajouter(p, Some(comp), &mut pages);
        }
    }

    // 2. SvelteKit : src/routes/**/+page.svelte.
    for f in fichiers
        .iter()
        .filter(|f| f.chemin.ends_with("/+page.svelte") && f.chemin.contains("routes/"))
    {
        let dossiers = f
            .chemin
            .split("routes/")
            .nth(1)
            .unwrap_or("")
            .trim_end_matches("+page.svelte")
            .trim_end_matches('/');
        let segments: Vec<&str> = dossiers
            .split('/')
            .filter(|s| !s.is_empty() && !(s.starts_with('(') && s.ends_with(')')))
            .collect();
        let route = route_normale(&segments.join("/"));
        let nom = if route == "/" {
            "Accueil".into()
        } else {
            humaniser(segments.last().unwrap_or(&""))
        };
        let symboles: Vec<String> = index
            .conn
            .prepare("SELECT id FROM symboles WHERE fichier = ?1 AND genre = 'composant'")
            .and_then(|mut st| st.query_map([&f.chemin], |r| r.get(0))?.collect())
            .unwrap_or_default();
        let p = Page {
            cle: route.clone(),
            nom,
            route: Some(route),
            exigences: exigences(index, &symboles),
            symboles,
            fichier: Some(f.chemin.clone()),
            source: "fichier-route".into(),
            groupe: None,
            masque: false,
            corrige: false,
            disparue: false,
        };
        ajouter(p, Some(f.chemin.clone()), &mut pages);
    }

    // 3. Énumération de navigation : enum Page { … } (Rust, TypeScript).
    for f in fichiers
        .iter()
        .filter(|f| matches!(f.ext.as_str(), "rs" | "ts" | "tsx"))
    {
        for m in ENUM.captures_iter(&f.texte) {
            let type_enum = m[1].to_string();
            let sep = if f.ext == "rs" { "::" } else { "." };
            let variantes: Vec<String> = m[2]
                .split(',')
                .map(|v| {
                    v.lines()
                        .filter(|l| {
                            !l.trim_start().starts_with("//") && !l.trim_start().starts_with('#')
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .filter_map(|v| {
                    v.split_whitespace().next().map(|x| {
                        x.chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect::<String>()
                    })
                })
                .filter(|v| v.chars().next().is_some_and(char::is_uppercase))
                .collect();
            for v in variantes {
                let reference = format!("{type_enum}{sep}{v}");
                let libelle =
                    Regex::new(&format!(r#"{}\s*,\s*"([^"]+)""#, regex::escape(&reference)))
                        .ok()
                        .and_then(|re| {
                            fichiers
                                .iter()
                                .find_map(|g| re.captures(&g.texte).map(|c| c[1].to_string()))
                        });
                let comp = composant_variante(index, &fichiers, &reference, &v);
                let nom = libelle.unwrap_or_else(|| humaniser(&v));
                let p = page(
                    index,
                    v.to_lowercase(),
                    nom,
                    None,
                    comp.as_deref(),
                    Some(f.chemin.clone()),
                    "enum",
                    None,
                );
                ajouter(p, comp, &mut pages);
            }
        }
    }

    // 4. Pages HTML (hors coquille d'une application monopage).
    for f in fichiers.iter().filter(|f| f.ext == "html") {
        let t = &f.texte;
        // Un fragment (gabarit Angular `*.component.html`) n'est pas une page ; une coquille
        // d'application monopage non plus (ses écrans sont trouvés par les autres détecteurs).
        let debut = t.chars().take(2000).collect::<String>().to_lowercase();
        if !(debut.contains("<html") || debut.contains("<!doctype")) {
            continue;
        }
        if [
            "<app-root",
            "id=\"app\"",
            "id=\"root\"",
            "type=\"module\"",
            "data-trunk",
            "%sveltekit",
        ]
        .iter()
        .any(|x| t.contains(x))
        {
            continue;
        }
        let nom = TITRE_HTML
            .captures(t)
            .map(|c| c[1].trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                humaniser(
                    f.chemin
                        .rsplit('/')
                        .next()
                        .unwrap_or("")
                        .trim_end_matches(".html"),
                )
            });
        let p = Page {
            cle: f.chemin.to_lowercase(),
            nom,
            route: Some(format!("/{}", f.chemin)),
            symboles: vec![],
            exigences: vec![],
            fichier: Some(f.chemin.clone()),
            source: "html".into(),
            groupe: f.chemin.rsplit_once('/').map(|(d, _)| d.to_string()),
            masque: false,
            corrige: false,
            disparue: false,
        };
        ajouter(p, Some(f.chemin.clone()), &mut pages);
    }

    // 5. Composants rangés dans un dossier de pages. Une page déjà trouvée qui a son propre dossier
    //    sous `pages/` (`pages/optim/optim.component.ts`) possède ce dossier : ses étapes et graphiques
    //    sont des morceaux de la page, pas des pages.
    let dossiers_de_pages: Vec<String> = pages
        .iter()
        .filter_map(|p| {
            p.fichier
                .as_deref()?
                .rsplit_once('/')
                .map(|(d, _)| d.to_string())
        })
        .filter(|d| {
            d.split('/')
                .next_back()
                .is_some_and(|dernier| !DOSSIERS_PAGES.contains(&dernier))
                && d.split('/').any(|x| DOSSIERS_PAGES.contains(&x))
        })
        .collect();
    let mut st = index.conn.prepare(
        "SELECT id, nom, fichier, genre FROM symboles WHERE est_test = 0 AND visibilite <> 'champ' ORDER BY fichier, ligne_debut",
    )?;
    let symboles: Vec<(String, String, String, String)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<rusqlite::Result<_>>()?;
    for (_id, nom, fichier, genre) in symboles {
        let dossiers: Vec<&str> = fichier.split('/').collect();
        let Some(pos) = dossiers[..dossiers.len().saturating_sub(1)]
            .iter()
            .rposition(|d| DOSSIERS_PAGES.contains(d))
        else {
            continue;
        };
        // Un composant porte un nom en majuscule : `allerPage()` est une méthode, pas une page.
        let majuscule = nom.chars().next().is_some_and(char::is_uppercase);
        let est_page = majuscule
            && (genre == "composant"
                || nom.ends_with("Page")
                || (genre == "class" && nom.ends_with("Component")));
        if dossiers_de_pages
            .iter()
            .any(|d| fichier.starts_with(&format!("{d}/")))
        {
            continue;
        }
        if !est_page || nom.starts_with('+') {
            continue;
        }
        let sous: Vec<&str> = dossiers[pos + 1..dossiers.len() - 1].to_vec();
        let groupe = if sous.is_empty() {
            Some(dossiers[pos].to_string())
        } else {
            Some(sous.join("/"))
        };
        let lisible = humaniser(nom.trim_end_matches("Component").trim_end_matches("Page"));
        let p = page(
            index,
            nom.to_lowercase(),
            lisible,
            None,
            Some(&nom),
            Some(fichier.clone()),
            "composant",
            groupe,
        );
        ajouter(p, Some(nom), &mut pages);
    }
    Ok(pages)
}

/// Composant d'une variante de navigation : d'abord un symbole qui porte son nom (`TemplatePage`,
/// `ImportCvSection`), sinon la première balise de composant qui suit la condition `… Page::X>`.
fn composant_variante(
    index: &Index,
    fichiers: &[Source],
    reference: &str,
    variante: &str,
) -> Option<String> {
    let suffixes = [
        "Page",
        "Section",
        "View",
        "Screen",
        "Ecran",
        "Vue",
        "Component",
    ];
    let par_nom: Option<String> = index
        .conn
        .prepare(
            "SELECT nom FROM symboles WHERE est_test = 0 AND nom LIKE ?1 ORDER BY length(nom), nom",
        )
        .ok()
        .and_then(|mut st| {
            let noms: Vec<String> = st
                .query_map([format!("{variante}%")], |r| r.get(0))
                .ok()?
                .flatten()
                .collect();
            noms.into_iter()
                .find(|n| suffixes.iter().any(|s| n.ends_with(s)))
        });
    if par_nom.is_some() {
        return par_nom;
    }
    for f in fichiers {
        let mut depart = 0;
        while let Some(i) = f.texte[depart..].find(reference) {
            let fin = depart + i + reference.len();
            depart = fin;
            let suite = &f.texte[fin..];
            // La condition d'affichage se termine par la balise ouvrante : `… != Page::X>`.
            if !suite.trim_start().starts_with('>') {
                continue;
            }
            let fenetre: String = suite.chars().take(400).collect();
            if let Some(c) = BALISE_MAJ
                .captures_iter(&fenetre)
                .map(|c| c[1].to_string())
                .find(|c| !STRUCTURE.contains(&c.as_str()))
            {
                return Some(c);
            }
        }
    }
    None
}

// ---------------------------------------------------------------- carte persistée

fn chemin_carte(index: &Index) -> String {
    format!(
        "{}/carte.md",
        index.config.reception.dossier.trim_end_matches('/')
    )
}

fn page_generale() -> Page {
    Page {
        cle: GENERAL.into(),
        nom: "Général (remarques transverses)".into(),
        route: None,
        symboles: vec![],
        exigences: vec![],
        fichier: None,
        source: "manuel".into(),
        groupe: None,
        masque: false,
        corrige: false,
        disparue: false,
    }
}

/// Carte enregistrée (`docs/reception/carte.md`), vide si le cahier n'a pas encore été scanné.
/// @relu 2026-10-18 bff989d0
pub fn lire(index: &Index) -> Vec<Page> {
    frontmatter_carte(index)
        .and_then(|v| v.get("pages").cloned())
        .and_then(|p| serde_json::from_value(p).ok())
        .unwrap_or_default()
}

fn frontmatter_carte(index: &Index) -> Option<Value> {
    let t = std::fs::read_to_string(index.racine.join(chemin_carte(index))).ok()?;
    let (yaml, _, _) = crate::projet::separer_frontmatter(&t);
    yaml.and_then(|y| serde_yaml_ng::from_str::<Value>(y).ok())
}

/// Adresse de l'application reçue (SPEC-23 R7), saisie en tête de la carte.
pub fn url_application(index: &Index) -> Option<String> {
    frontmatter_carte(index)?
        .get("url_application")?
        .as_str()
        .map(str::to_string)
}

/// Enregistre l'adresse de l'application reçue (vide : retirée).
pub fn definir_url_application(index: &Index, url: &str) -> Result<()> {
    let url = url.trim().trim_end_matches('/');
    if !url.is_empty() && !(url.starts_with("http://") || url.starts_with("https://")) {
        anyhow::bail!("adresse attendue en http:// ou https:// : {url}");
    }
    ecrire_avec(
        index,
        &lire(index),
        Some(url).filter(|u| !u.is_empty()),
        None,
    )
}

/// Dernier numéro de remarque attribué : un numéro n'est jamais réutilisé, même après suppression.
pub fn dernier_numero(index: &Index) -> u32 {
    frontmatter_carte(index)
        .and_then(|v| v.get("derniere_remarque")?.as_u64())
        .map_or(0, |n| n as u32)
}

/// Note le dernier numéro de remarque attribué.
pub fn noter_numero(index: &Index, n: u32) -> Result<()> {
    if n > dernier_numero(index) {
        ecrire_avec(
            index,
            &lire(index),
            url_application(index).as_deref(),
            Some(n),
        )?;
    }
    Ok(())
}

/// Écrit la carte (frontmatter : les pages ; corps : un tableau lisible) ; l'adresse de l'application
/// déjà saisie est gardée.
pub fn ecrire(index: &Index, pages: &[Page]) -> Result<()> {
    ecrire_avec(index, pages, url_application(index).as_deref(), None)
}

fn ecrire_avec(
    index: &Index,
    pages: &[Page],
    url: Option<&str>,
    numero: Option<u32>,
) -> Result<()> {
    let numero = numero.unwrap_or_else(|| dernier_numero(index));
    let mut fm = Map::new();
    fm.insert("id".into(), json!("CARTE-RECEPTION"));
    fm.insert("type".into(), json!("carte-reception"));
    fm.insert("titre".into(), json!("Carte des pages"));
    fm.insert("statut".into(), json!("brouillon"));
    if let Some(u) = url {
        fm.insert("url_application".into(), json!(u));
    }
    if numero > 0 {
        fm.insert("derniere_remarque".into(), json!(numero));
    }
    fm.insert("pages".into(), serde_json::to_value(pages)?);
    let mut corps = String::from(
        "# Carte des pages\n\n> Construite par `wikik reception` à partir du code ; noms, masques et groupes corrigés à l'écran sont gardés par les scans suivants.\n\n| Page | Route | Code | Source |\n|---|---|---|---|\n",
    );
    for p in pages.iter().filter(|p| !p.masque) {
        corps.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            p.nom,
            p.route.as_deref().unwrap_or(""),
            p.symboles
                .first()
                .map(|s| format!("`{s}`"))
                .unwrap_or_default(),
            p.source
        ));
    }
    let chemin = index.racine.join(chemin_carte(index));
    std::fs::create_dir_all(chemin.parent().unwrap())?;
    std::fs::write(chemin, crate::atelier::artefacts::formater(&fm, &corps))?;
    Ok(())
}

/// Fusionne un scan avec la carte existante (SPEC-22 R4) : corrections gardées (nom, masque, groupe,
/// route ajoutée à la main), nouvelles pages ajoutées, pages disparues marquées.
pub fn fusionner(anciennes: &[Page], trouvees: Vec<Page>) -> Vec<Page> {
    let par_cle: BTreeMap<&str, &Page> = anciennes.iter().map(|p| (p.cle.as_str(), p)).collect();
    let mut sortie: Vec<Page> = vec![];
    let mut vues = BTreeSet::new();
    for mut p in trouvees {
        if let Some(a) = par_cle.get(p.cle.as_str())
            && a.corrige
        {
            p.nom = a.nom.clone();
            p.masque = a.masque;
            if a.route.is_some() {
                p.route = a.route.clone();
            }
            p.groupe = a.groupe.clone();
            p.corrige = true;
        }
        vues.insert(p.cle.clone());
        sortie.push(p);
    }
    // Pages ajoutées à la main ou par le LLM, et pages qui ne sont plus trouvées : gardées.
    for a in anciennes.iter().filter(|a| !vues.contains(&a.cle)) {
        let mut a = a.clone();
        if !matches!(a.source.as_str(), "manuel" | "llm") {
            a.disparue = true;
        }
        sortie.push(a);
    }
    // La page des remarques transverses est toujours la première.
    let generale = match sortie.iter().position(|p| p.cle == GENERAL) {
        Some(i) => sortie.remove(i),
        None => page_generale(),
    };
    sortie.insert(0, generale);
    sortie
}

/// Scanne le projet, fusionne avec la carte existante et l'écrit ; renvoie la carte.
pub fn scanner(index: &Index) -> Result<Vec<Page>> {
    let pages = fusionner(&lire(index), detecter(index)?);
    ecrire(index, &pages)?;
    Ok(pages)
}

/// Corrige une page (nom, masque, groupe, route) ; la correction est marquée pour les scans suivants.
pub fn corriger(
    index: &Index,
    cle: &str,
    nom: Option<&str>,
    masque: Option<bool>,
    groupe: Option<&str>,
    route: Option<&str>,
) -> Result<Page> {
    let mut pages = lire(index);
    let p = pages
        .iter_mut()
        .find(|p| p.cle == cle)
        .ok_or_else(|| anyhow::anyhow!("page inconnue : {cle}"))?;
    if let Some(n) = nom.map(str::trim).filter(|n| !n.is_empty()) {
        p.nom = n.to_string();
    }
    if let Some(m) = masque {
        p.masque = m;
    }
    if let Some(g) = groupe {
        p.groupe = Some(g.trim().to_string()).filter(|g| !g.is_empty());
    }
    // Route ajoutée ou corrigée à la main : sert au lien « ouvrir dans l'application » (SPEC-23 R7).
    if let Some(r) = route {
        p.route = Some(r.trim()).filter(|r| !r.is_empty()).map(|r| {
            if r.starts_with('/') {
                r.to_string()
            } else {
                format!("/{r}")
            }
        });
    }
    p.corrige = true;
    let copie = p.clone();
    ecrire(index, &pages)?;
    Ok(copie)
}

/// Ajoute des pages proposées par le LLM (SPEC-22 R5) : seules celles dont le symbole existe sont gardées.
pub fn ajouter_llm(index: &Index, propositions: &[Value]) -> Result<Vec<Page>> {
    let mut pages = lire(index);
    let mut ajoutees = vec![];
    for e in propositions {
        let t = |k: &str| {
            e.get(k)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string()
        };
        let sym = t("symbole");
        let Ok((id, fichier)) = index.conn.query_row(
            "SELECT id, fichier FROM symboles WHERE id = ?1 OR nom = ?1 LIMIT 1",
            [&sym],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        ) else {
            continue;
        };
        let route = Some(t("route"))
            .filter(|r| !r.is_empty())
            .map(|r| route_normale(&r));
        let cle = route.clone().unwrap_or_else(|| t("nom").to_lowercase());
        if cle.is_empty()
            || pages
                .iter()
                .any(|p| p.cle == cle || p.symboles.contains(&id))
        {
            continue;
        }
        let symboles = vec![id];
        let p = Page {
            cle,
            nom: t("nom"),
            route,
            exigences: exigences(index, &symboles),
            symboles,
            fichier: Some(fichier),
            source: "llm".into(),
            groupe: None,
            masque: false,
            corrige: false,
            disparue: false,
        };
        ajoutees.push(p.clone());
        pages.push(p);
    }
    ecrire(index, &pages)?;
    Ok(ajoutees)
}

/// Demande au LLM les pages que les détecteurs n'ont pas trouvées (SPEC-22 R5) : (consigne, schéma).
pub fn demande_completion(index: &Index) -> (String, Value) {
    let connues: Vec<String> = lire(index)
        .iter()
        .filter(|p| p.cle != GENERAL)
        .map(|p| {
            format!(
                "- {} {} {}",
                p.nom,
                p.route.as_deref().unwrap_or(""),
                p.symboles.first().map(String::as_str).unwrap_or("")
            )
        })
        .collect();
    let consigne = format!(
        "Tu prépares la réception d'une application : la liste de ses écrans, pour que l'utilisateur y dépose ses remarques.\n\
         Lis le projet avec les outils wikik (carte, fichier, symbole, cherche, pages) ; tu n'écris rien.\n\n\
         Écrans déjà trouvés par l'analyse du code :\n{}\n\n\
         Trouve les écrans que l'utilisateur peut atteindre et qui manquent (vues affichées par un état, onglets, \
         fenêtres modales importantes, pages servies par le serveur). Pour chacun : un nom court en français, sa route \
         si elle existe, l'identifiant exact du symbole de l'index qui l'affiche (`chemin::Nom`), et en une phrase la \
         raison. N'invente aucun symbole : s'il n'est pas dans l'index, n'ajoute pas l'écran. Réponds `elements: []` \
         si rien ne manque.",
        if connues.is_empty() {
            "(aucun)".to_string()
        } else {
            connues.join("\n")
        }
    );
    let schema = json!({
        "type": "object", "required": ["elements"],
        "properties": { "elements": { "type": "array", "items": {
            "type": "object", "required": ["nom", "symbole", "raison"],
            "properties": { "nom": {"type": "string"}, "route": {"type": "string"}, "symbole": {"type": "string"}, "raison": {"type": "string"} }
        }}}
    });
    (consigne, schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noms_lisibles() {
        assert_eq!(humaniser("TemplatePage"), "Template page");
        assert_eq!(humaniser("search-optim"), "Search optim");
        assert_eq!(route_normale("configurer"), "/configurer");
        assert_eq!(route_normale(""), "/");
    }

    #[test]
    fn fusion_garde_les_corrections() {
        let mk = |cle: &str, nom: &str| Page {
            cle: cle.into(),
            nom: nom.into(),
            route: None,
            symboles: vec![],
            exigences: vec![],
            fichier: None,
            source: "route".into(),
            groupe: None,
            masque: false,
            corrige: false,
            disparue: false,
        };
        let mut corrigee = mk("/a", "Mon nom");
        corrigee.corrige = true;
        corrigee.masque = true;
        let anciennes = vec![corrigee, mk("/b", "B")];
        let r = fusionner(&anciennes, vec![mk("/a", "A"), mk("/c", "C")]);
        let a = r.iter().find(|p| p.cle == "/a").unwrap();
        assert!(a.nom == "Mon nom" && a.masque);
        assert!(r.iter().find(|p| p.cle == "/b").unwrap().disparue);
        assert!(r.iter().any(|p| p.cle == "/c") && r[0].cle == GENERAL);
    }
}
