//! `wikik relu` : marque le bloc d'un symbole comme relu en ajoutant ou mettant à jour `@relu AAAA-MM-JJ`.
//!
//! C'est la seule écriture de wikik dans le code (SPEC-04) : toucher le bloc le rend frais
//! pour la détection de doc périmée, sans en réécrire le texte.
//!
//! @req EF-015
//! @adr 6

use crate::index::Index;
use anyhow::{Context, Result, bail};

/// Date du jour (UTC) au format `AAAA-MM-JJ`, sans dépendance de calendrier.
pub fn aujourdhui() -> String {
    let secondes = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    date_civile((secondes / 86_400) as i64)
}

/// Jours depuis le 1970-01-01 → date grégorienne (algorithme de H. Hinnant).
fn date_civile(jours: i64) -> String {
    let z = jours + 719_468;
    let ere = z.div_euclid(146_097);
    let jde = z.rem_euclid(146_097);
    let ade = (jde - jde / 1460 + jde / 36_524 - jde / 146_096) / 365;
    let a = ade + ere * 400;
    let jda = jde - (365 * ade + ade / 4 - ade / 100);
    let mp = (5 * jda + 2) / 153;
    let j = jda - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{:02}-{:02}", if m <= 2 { a + 1 } else { a }, m, j)
}

/// Empreinte courte du code relu (signature + corps), écrite après la date de `@relu` : le texte du
/// bloc change donc à chaque relecture d'un code différent, et `check` sait quel code a été relu.
pub fn jeton(hash_signature: &str, hash_corps: &str) -> String {
    crate::hash::empreinte(format!("{hash_signature}{hash_corps}").as_bytes())[..8].to_string()
}

/// Ajoute ou remplace `@relu <date> <jeton>` à la fin du bloc de `id`, dans le format du bloc
/// (`///`, `/** */` ou `//`) ; renvoie le fichier modifié.
pub fn marquer(index: &Index, id: &str, date: &str) -> Result<String> {
    let (fichier, debut, fin, corps, sig): (String, Option<i64>, Option<i64>, String, String) = index
        .conn
        .query_row(
            "SELECT fichier, doc_ligne_debut, doc_ligne_fin, hash_corps, hash_signature FROM symboles WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .with_context(|| format!("symbole inconnu : {id}"))?;
    let (Some(debut), Some(fin)) = (debut, fin) else {
        bail!("{id} n'a pas de bloc de doc à marquer")
    };
    let chemin = index.racine.join(&fichier);
    let texte = std::fs::read_to_string(&chemin)?;
    let fin_ligne = if texte.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lignes: Vec<String> = texte.lines().map(str::to_string).collect();
    let (d, f) = (debut as usize - 1, fin as usize - 1);
    poser_relu(
        &mut lignes,
        d,
        f,
        &format!("@relu {date} {}", jeton(&sig, &corps)),
    )
    .with_context(|| format!("{id} : bloc de doc dans un format non reconnu"))?;
    let mut sortie = lignes.join(fin_ligne);
    if texte.ends_with('\n') {
        sortie.push_str(fin_ligne);
    }
    std::fs::write(&chemin, sortie)?;
    Ok(fichier)
}

/// Écrit `balise` dans le bloc des lignes `d..=f` en respectant son format : `///` (Rust),
/// `/** … */` (JSDoc, éventuellement sur une seule ligne) ou `//` (en-tête de script).
fn poser_relu(lignes: &mut Vec<String>, d: usize, f: usize, balise: &str) -> Option<()> {
    let indentation = |l: &str| {
        l.chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>()
    };
    // Une ligne `@relu` existe déjà : on remplace ce qui suit `@relu`.
    if let Some(i) = (d..=f).find(|&i| lignes[i].contains("@relu")) {
        let l = &lignes[i];
        let p = l.find("@relu")?;
        let fin_jsdoc = if l.trim_end().ends_with("*/") {
            " */"
        } else {
            ""
        };
        lignes[i] = format!("{}{balise}{fin_jsdoc}", &l[..p]);
        return Some(());
    }
    let derniere = lignes[f].clone();
    let t = derniere.trim_start();
    let ind = indentation(&derniere);
    if t.starts_with("///") {
        lignes.insert(f + 1, format!("{ind}/// {balise}"));
    } else if let Some(p) = derniere.rfind("*/") {
        let avant = derniere[..p].trim_end();
        let ind_bloc = indentation(&lignes[d]);
        if avant.trim().is_empty() || avant.trim() == "*" {
            // Fermeture seule sur sa ligne : la balise juste avant.
            lignes.insert(f, format!("{ind_bloc} * {balise}"));
        } else {
            // `/** Texte. */` ou `… texte */` : on ouvre le bloc sur plusieurs lignes.
            lignes[f] = avant.to_string();
            lignes.insert(f + 1, format!("{ind_bloc} * {balise}"));
            lignes.insert(f + 2, format!("{ind_bloc} */"));
        }
    } else if t.starts_with("//") {
        lignes.insert(f + 1, format!("{ind}// {balise}"));
    } else {
        return None;
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::poser_relu;

    fn poser(texte: &str, d: usize, f: usize) -> String {
        let mut l: Vec<String> = texte.lines().map(str::to_string).collect();
        poser_relu(&mut l, d, f, "@relu 2026-01-01 abcd1234").unwrap();
        l.join("\n")
    }

    #[test]
    fn formats_de_bloc() {
        assert_eq!(
            poser("/// Doc.\nfn f() {}", 0, 0),
            "/// Doc.\n/// @relu 2026-01-01 abcd1234\nfn f() {}"
        );
        assert_eq!(
            poser("  /** Doc. */\n  f() {}", 0, 0),
            "  /** Doc.\n   * @relu 2026-01-01 abcd1234\n   */\n  f() {}"
        );
        assert_eq!(
            poser("/**\n * Doc.\n */\nfunction f() {}", 0, 2),
            "/**\n * Doc.\n * @relu 2026-01-01 abcd1234\n */\nfunction f() {}"
        );
        assert_eq!(
            poser("  // Rôle.\n  import x;", 0, 0),
            "  // Rôle.\n  // @relu 2026-01-01 abcd1234\n  import x;"
        );
        assert_eq!(
            poser("/// Doc.\n/// @relu 2025-01-01 00000000\nfn f() {}", 0, 1),
            "/// Doc.\n/// @relu 2026-01-01 abcd1234\nfn f() {}"
        );
        assert_eq!(
            poser("/** Doc. @relu 2025-01-01 00000000 */", 0, 0),
            "/** Doc. @relu 2026-01-01 abcd1234 */"
        );
    }

    #[test]
    fn dates() {
        assert_eq!(super::date_civile(0), "1970-01-01");
        assert_eq!(super::date_civile(20_743), "2026-10-17");
        assert_eq!(super::date_civile(11_016), "2000-02-29");
    }
}
