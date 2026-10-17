//! `wikik stats` : chiffres de couverture de la doc, pour un tableau de bord ou une tendance en CI.
//!
//! @req EF-085

use crate::check;
use crate::index::Index;
use anyhow::Result;
use serde::Serialize;

/// Couverture de la doc d'un projet.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Stats {
    pub fichiers: u32,
    /// Symboles hors tests et hors champs de classe (ceux pour lesquels un bloc a un sens).
    pub symboles: u32,
    pub documentes: u32,
    /// Part documentée, en pourcentage entier.
    pub taux: u32,
    pub perimes: u32,
    pub manquants: u32,
    pub documents: u32,
    pub exigences_must: u32,
    pub exigences_must_couvertes: u32,
}

/// Calcule les chiffres sur un index à jour ; les périmés et manquants viennent de `check`.
pub fn calculer(index: &Index) -> Result<Stats> {
    let c = &index.conn;
    let n = |sql: &str| -> Result<u32> { Ok(c.query_row(sql, [], |r| r.get::<_, i64>(0))? as u32) };
    let fichiers = n("SELECT COUNT(*) FROM fichiers")?;
    let symboles = n("SELECT COUNT(*) FROM symboles WHERE est_test = 0 AND genre <> 'champ'")?;
    let documentes = n(
        "SELECT COUNT(*) FROM symboles WHERE est_test = 0 AND genre <> 'champ' AND doc IS NOT NULL",
    )?;
    let documents = n("SELECT COUNT(*) FROM documents")?;
    let exigences_must = n(
        "SELECT COUNT(*) FROM documents WHERE genre IN ('exigence', 'exigence-nf') AND priorite = 'must'",
    )?;
    let exigences_must_couvertes = n(
        "SELECT COUNT(*) FROM documents d WHERE d.genre IN ('exigence', 'exigence-nf') AND d.priorite = 'must'
         AND EXISTS (SELECT 1 FROM aretes a WHERE a.cible = d.id AND a.genre = 'req')",
    )?;
    let rapport = check::verifier(index)?;
    let compter =
        |f: &dyn Fn(&str) -> bool| rapport.diagnostics.iter().filter(|d| f(&d.code)).count() as u32;
    Ok(Stats {
        fichiers,
        symboles,
        documentes,
        taux: (documentes * 100).checked_div(symboles).unwrap_or(0),
        perimes: compter(&|c| c == "perime-corps" || c == "perime-signature"),
        manquants: compter(&|c| c == "manquant"),
        documents,
        exigences_must,
        exigences_must_couvertes,
    })
}

/// Texte court, une mesure par ligne.
pub fn formater(s: &Stats) -> String {
    format!(
        "fichiers               {}\nsymboles               {}\ndocumentés             {} ({} %)\ndoc périmée            {}\ndoc manquante          {}\ndocuments Projet       {}\nexigences must         {} couvertes sur {}\n",
        s.fichiers,
        s.symboles,
        s.documentes,
        s.taux,
        s.perimes,
        s.manquants,
        s.documents,
        s.exigences_must_couvertes,
        s.exigences_must
    )
}
