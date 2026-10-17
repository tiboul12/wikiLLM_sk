---
id: CDC-v3
type: cahier-des-charges
titre: "Cahier des charges v3 (équipe)"
statut: proposé
version: v3
sources: [BDR-v3-2]
---

# Cahier des charges v3

## Exigences fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/EF-080]] | Exporter le wiki en site statique | must | B-011 |
| [[exigences/EF-081]] | Annoter les PR avec les diagnostics | must | B-011 |
| [[exigences/EF-082]] | Générer la CI et les validateurs | must | B-011 |
| [[exigences/EF-083]] | Installer un hook pre-commit | should | B-011 |
| [[exigences/EF-084]] | Créer un identifiant | should | B-011 |
| [[exigences/EF-085]] | Mesurer la couverture | should | B-011 |

## Exigences non fonctionnelles

ENF-002 (déterminisme) et ENF-004 (sécurité) s'appliquent au site exporté : mêmes données pour un même commit, aucune écriture possible, aucune ressource externe.

## Exigences modifiées ou abandonnées

Aucune.
