---
name: wikik-structure-adr
description: Comment rédiger une décision d'architecture (ADR) de wikik — contexte, décision, conséquences, alternatives écartées. Utilisé quand le LLM ou l'agent propose ou corrige un ADR.
---

# Structure d'un ADR

Un fichier par décision : `docs/projet/adr/NNNN-titre.md`.

## Frontmatter

`id: ADR-NNNN`, `type: adr`, `titre` (la décision en une phrase), `statut` (`proposé`, `accepté`, `remplacé`), `date`, `exigences: [EF-xxx]`.

## Corps

1. **Contexte** : la situation et la contrainte qui forcent un choix (avec preuves).
2. **Décision** : ce qui est retenu, en règles vérifiables.
3. **Conséquences** : ce qui devient plus simple, plus difficile, ce qu'il faudra surveiller.
4. **Alternatives écartées** : chacune avec la raison.

Le code qui applique la décision porte `@adr N`.
