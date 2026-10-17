---
id: MESURE-01
type: mesure
titre: "Coût en tokens du contexte, avec et sans wikik (ATS)"
statut: terminé
version: v1
date: 2026-10-17
sources: [SPEC-07, ENF-001, BDR-v1-2]
---

# MESURE-01 : coût du contexte avec et sans wikik, sur ATS

**Terrain** : clone d'ATS au 17/10/2026 (29 fichiers Rust, 11 163 lignes, environ 117 000 tokens de code au total).
**Unité** : tokens estimés = octets / 4, la même règle que l'en-tête des requêtes.
**Nature de la mesure** : **approximation**. « Sans wikik » = les fichiers qu'il faudrait lire en entier pour réunir le même contexte (le symbole et tous ses appelants). Un agent réel utilise aussi `grep` et des lectures partielles : le gain réel est donc plus faible que ce tableau. La mesure en session réelle reste à faire (protocole plus bas).

## Résultats

| Tâche | Avec wikik (requêtes) | Sans wikik (fichiers à lire) | Rapport |
|---|---|---|---|
| T1 Changer la règle de `flow_paragraphs` (qui l'utilise ?) | 292 (`symbole` + `impact`) | 24 638 (`shared/src/lib.rs`, `cv_render.rs`, `template_page.rs`) | ÷ 84 |
| T2 Changer la signature de `compile` | 1 116 (`symbole` + `impact`) | 24 485 (`compiler.rs`, `lib.rs`, `cv_render.rs`, `validator.rs`) | ÷ 22 |
| T3 Accepter un nouveau format à l'import | 963 (`cherche` + `symbole --code` + `impact`) | 28 764 (`src/app.rs`) | ÷ 30 |
| T4 Vue d'ensemble en début de session | 1 098 (`carte`) | 41 659 (README + 3 fichiers principaux) | ÷ 38 |

## Budgets de SPEC-07 (ENF-001)

| Requête | Mesuré sur ATS | Budget |
|---|---|---|
| `carte` | 1 098 | ≤ 4 000 ✅ |
| `fichier src/app.rs` (2 521 lignes, 76 symboles) | 807 | ≤ 1 500 ✅ |
| `fichier shared/src/lib.rs` (68 symboles, surtout publics) | ≈ 1 500 | ≤ 1 500 ✅ (limite) |
| `symbole compile` | 400 | ≤ 800 ✅ |
| `impact compile` (13 appelants) | 716 | ≤ 1 500 ✅ |
| `cherche photo recadrage` | 336 | ≤ 1 500 ✅ |

## Ce que la mesure a fait corriger

- `impact flow_paragraphs` ne trouvait aucun appelant : ATS l'appelle par `ats_cv_shared::flow_paragraphs`, le nom de sa crate locale. L'index lit maintenant les `Cargo.toml` du projet (+13 appels certains sur ATS).
- `fichier src/app.rs` coûtait environ 2 000 tokens : au-delà de 40 symboles, les privés sont listés par leur nom seulement.

## Protocole de mesure réelle (à faire par Tibo)

1. Choisir 3 à 5 tâches réelles d'ATS.
2. Les faire dans deux sessions Claude Code neuves : sans le skill wikik, puis avec.
3. Relever le nombre de tokens d'entrée de la session (`/cost` ou l'interface) et le nombre de fichiers source ouverts.
4. Consigner les résultats ici, dans une section « Mesure réelle ».
