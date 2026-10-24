---
name: wikik-structure-spec
description: Comment rédiger une spécification (SPEC-xx) de wikik — règles numérotées et critères d'acceptation, écrite juste avant sa phase. Utilisé quand le LLM ou l'agent produit ou corrige une spec.
---

# Structure d'une spec

Un fichier par spec : `docs/projet/specs/SPEC-xx-titre.md`, écrite juste avant la phase qui l'implémente.

## Frontmatter

`id`, `type: spec`, `titre`, `statut`, `version`, `exigences: [EF-xxx]`.

## Corps

- **Objet** : ce que la spec couvre, en une phrase.
- **Règles** `R1`, `R2`… : comportements vérifiables, une règle par point.
- **Critères d'acceptation** : les scénarios qui prouvent les règles.
- **Hors sujet** : ce qui n'est pas couvert.
