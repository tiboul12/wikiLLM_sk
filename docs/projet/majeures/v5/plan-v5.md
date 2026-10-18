---
id: PLAN-v5
type: plan
titre: "Plan v5 : cahier de réception"
statut: validé
version: v5
sources: [BDR-v5-2, CDC-v5]
---

# Plan v5 : cahier de réception

**En une phrase** : `wikik reception`, dans n'importe quel projet, construit la carte des pages à partir du code et recueille les remarques de l'utilisateur page par page, images comprises ; l'agent de code les traite ensuite par MCP.

## Phases

| # | Phase | Sortie |
|---|---|---|
| 0 | BDR, cahier des charges, SPEC-22 et 23 | documents `proposé` |
| 1 | Carte des pages : détecteurs, fusion, LLM | fixture + wikik, ATS (clone), shelley |
| 2 | Remarques, images, index, serveur `wikik reception`, MCP | tests HTTP et MCP |
| 3 | Écrans : carte, page, traitement | contrôle headless clair / sombre / téléphone |
| 4 | Essai sur wikik lui-même, clôture | tag `v5.0-rc1` |

## Suivi

| Phase | État | Date |
|---|---|---|
| 0 BDR, CDC, specs | ✅ | 2026-10-18 |
| 1 Carte des pages | ✅ 5 détecteurs (routes Angular / Vue / React / Leptos, SvelteKit, `enum Page`, HTML, composants sous `pages/`), fusion avec corrections ; `wikik query pages` ; wikik 9 pages, ATS 6 avec libellés, shelley-studio-llm 9 routes + `Benchmark` sans route, shelley-administration 8, shelley-accueil 2 | 2026-10-18 |
| 2 à 4 | ⏳ | |
