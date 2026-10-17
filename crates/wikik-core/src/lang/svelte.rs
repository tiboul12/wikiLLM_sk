//! Extraction Svelte : le script (`<script lang="ts">`) passe par l'extraction TypeScript ; le fichier
//! lui-même devient un symbole `composant`, relié aux composants qu'il rend dans son balisage.
//!
//! Tout ce qui est hors des balises `<script>` est remplacé par des espaces (retours à la ligne
//! gardés) : les numéros de ligne et de colonne du script restent exacts sans calcul de décalage.
//!
//! @req EF-070, EF-071
//! @adr 2

use super::Langage;
use super::rust::{balises, resume};
use super::typescript::{appels_niveau_module, extraire_ts, hash_normalise, lignes_jsdoc};
use crate::hash::{empreinte, empreinte_morceaux};
use crate::modele::{AppelBrut, ExtraitFichier, Symbole};
use tree_sitter::Parser;

/// Langage Svelte (composants `.svelte`).
pub struct Svelte;

impl Langage for Svelte {
    fn nom(&self) -> &'static str {
        "svelte"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["svelte"]
    }

    fn extraire(&self, chemin: &str, source: &[u8]) -> ExtraitFichier {
        let texte = String::from_utf8_lossy(source);
        let (script, balisage, en_ts) = separer(&texte);
        let grammaire: tree_sitter::Language = if en_ts {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        };
        let mut extrait = extraire_ts(chemin, script.as_bytes(), &grammaire);
        extrait.lignes = texte.lines().count();

        let tige = chemin
            .rsplit('/')
            .next()
            .unwrap_or(chemin)
            .trim_end_matches(".svelte")
            .to_string();
        let mut appels = appels_niveau_module(script.as_bytes(), &grammaire);
        appels.extend(composants_rendus(&balisage));
        appels.sort_by(|a, b| a.ligne.cmp(&b.ligne).then(a.cible.cmp(&b.cible)));
        let (doc, doc_debut, doc_fin) = match doc_composant(&texte) {
            (d, _, _) if d.is_empty() => commentaire_de_tete(&script),
            trouve => trouve,
        };
        let a_doc = !doc.is_empty();
        let doc_normalisee = doc.iter().map(|l| l.trim()).collect::<Vec<_>>().join("\n");
        let corps_script = {
            let mut p = Parser::new();
            p.set_language(&grammaire)
                .ok()
                .and_then(|_| p.parse(script.as_bytes(), None))
                .map(|a| hash_normalise(a.root_node(), script.as_bytes(), None))
        };
        let balisage_compact = balisage.split_whitespace().collect::<Vec<_>>().join(" ");
        extrait.symboles.insert(
            0,
            Symbole {
                id: format!("{chemin}::{tige}"),
                nom: tige.clone(),
                nom_qualifie: tige.clone(),
                genre: "composant".into(),
                visibilite: "pub".into(),
                ligne_debut: 1,
                ligne_fin: extrait.lignes.max(1),
                doc_ligne_debut: doc_debut,
                doc_ligne_fin: doc_fin,
                signature: format!("<{tige}> ({tige}.svelte)"),
                resume: resume(&doc),
                doc: a_doc.then(|| doc.join("\n")),
                hash_corps: empreinte_morceaux([
                    corps_script.unwrap_or_default().as_str(),
                    balisage_compact.as_str(),
                ]),
                hash_signature: String::new(),
                hash_doc: a_doc.then(|| empreinte(doc_normalisee.as_bytes())),
                est_test: false,
                type_parent: None,
                balises: balises(&doc, doc_debut),
                cfg: None,
                appels,
            },
        );
        extrait
    }
}

/// (script masqué, balisage masqué, script en TypeScript ?) : chacun garde la géométrie du fichier.
fn separer(texte: &str) -> (String, String, bool) {
    let mut script = String::with_capacity(texte.len());
    let mut balisage = String::with_capacity(texte.len());
    let mut en_ts = false;
    let mut reste = texte;
    let masquer = |t: &str| {
        t.chars()
            .map(|c| if c == '\n' { '\n' } else { ' ' })
            .collect::<String>()
    };
    while let Some(debut) = reste.find("<script") {
        let Some(fin_ouvrante) = reste[debut..].find('>').map(|i| debut + i + 1) else {
            break;
        };
        let ouvrante = &reste[debut..fin_ouvrante];
        en_ts |= ouvrante.contains("lang=\"ts\"")
            || ouvrante.contains("lang='ts'")
            || ouvrante.contains("lang=\"typescript\"");
        let fin_script = reste[fin_ouvrante..]
            .find("</script>")
            .map_or(reste.len(), |i| fin_ouvrante + i);
        // Avant le script : balisage ; balise ouvrante : masquée partout ; contenu : script.
        balisage.push_str(&reste[..debut]);
        script.push_str(&masquer(&reste[..fin_ouvrante]));
        balisage.push_str(&masquer(&reste[debut..fin_script]));
        script.push_str(&reste[fin_ouvrante..fin_script]);
        reste = &reste[fin_script..];
    }
    balisage.push_str(reste);
    script.push_str(&masquer(reste));
    (script, balisage, en_ts)
}

/// `<Composant …>` dans le balisage : arêtes « rend » (même heuristique que `view!` en Rust).
fn composants_rendus(balisage: &str) -> Vec<AppelBrut> {
    let mut v = vec![];
    for (i, ligne) in balisage.lines().enumerate() {
        let mut reste = ligne;
        while let Some(p) = reste.find('<') {
            let apres = &reste[p + 1..];
            let nom: String = apres
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if nom.chars().next().is_some_and(|c| c.is_uppercase()) {
                v.push(AppelBrut {
                    forme: "rend".into(),
                    cible: nom.clone(),
                    qualificatif: None,
                    ligne: i + 1,
                });
            }
            reste = &apres[nom.len().min(apres.len())..];
        }
    }
    v
}

/// Doc d'un composant : le commentaire HTML `<!-- @component … -->`, convention de Svelte.
fn doc_composant(texte: &str) -> (Vec<String>, Option<usize>, Option<usize>) {
    let Some(debut) = texte.find("<!-- @component") else {
        return (vec![], None, None);
    };
    let Some(fin) = texte[debut..].find("-->").map(|i| debut + i) else {
        return (vec![], None, None);
    };
    let contenu = &texte[debut + "<!-- @component".len()..fin];
    let ligne_debut = texte[..debut].matches('\n').count() + 1;
    let ligne_fin = texte[..fin].matches('\n').count() + 1;
    let lignes = lignes_jsdoc(&format!("/**{contenu}*/"));
    (lignes, Some(ligne_debut), Some(ligne_fin))
}

/// Sans `@component` : les commentaires `//` en tête du script décrivent le composant.
fn commentaire_de_tete(script: &str) -> (Vec<String>, Option<usize>, Option<usize>) {
    let mut lignes = vec![];
    let (mut debut, mut fin) = (None, None);
    for (i, l) in script.lines().enumerate() {
        let t = l.trim();
        if t.is_empty() && debut.is_none() {
            continue;
        }
        match t.strip_prefix("//") {
            Some(r) => {
                debut.get_or_insert(i + 1);
                fin = Some(i + 1);
                lignes.push(r.strip_prefix(' ').unwrap_or(r).trim_end().to_string());
            }
            None => break,
        }
    }
    (lignes, debut, fin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masquage_garde_les_lignes() {
        let t = "<script lang=\"ts\">\n  let a = 1;\n</script>\n\n<Cellule x={a} />\n";
        let (script, balisage, ts) = separer(t);
        assert!(ts);
        assert_eq!(script.lines().count(), t.lines().count());
        assert!(script.contains("let a = 1;") && !script.contains("Cellule"));
        assert!(balisage.contains("<Cellule") && !balisage.contains("let a"));
        let rendus = composants_rendus(&balisage);
        assert_eq!((rendus[0].cible.as_str(), rendus[0].ligne), ("Cellule", 5));
    }
}
