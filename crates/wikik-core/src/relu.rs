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

/// Ajoute ou remplace la ligne `@relu <date> <jeton>` à la fin du bloc de `id` ; renvoie le fichier modifié.
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
    let existante = (d..=f).find(|&i| {
        lignes[i]
            .trim_start()
            .trim_start_matches("///")
            .trim_start()
            .starts_with("@relu")
    });
    match existante {
        Some(i) => {
            let indentation: String = lignes[i]
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            lignes[i] = format!("{indentation}/// @relu {date} {}", jeton(&sig, &corps));
        }
        None => {
            if !lignes[f].trim_start().starts_with("///") {
                bail!(
                    "{id} : bloc `/** */` non pris en charge par relu, ajouter `@relu {date}` à la main"
                );
            }
            let indentation: String = lignes[f]
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            lignes.insert(
                f + 1,
                format!("{indentation}/// @relu {date} {}", jeton(&sig, &corps)),
            );
        }
    }
    let mut sortie = lignes.join(fin_ligne);
    if texte.ends_with('\n') {
        sortie.push_str(fin_ligne);
    }
    std::fs::write(&chemin, sortie)?;
    Ok(fichier)
}

#[cfg(test)]
mod tests {
    #[test]
    fn dates() {
        assert_eq!(super::date_civile(0), "1970-01-01");
        assert_eq!(super::date_civile(20_743), "2026-10-17");
        assert_eq!(super::date_civile(11_016), "2000-02-29");
    }
}
