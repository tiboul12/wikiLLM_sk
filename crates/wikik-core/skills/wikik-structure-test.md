---
name: wikik-structure-test
description: Comment rédiger un test d'exigence (TEST-xxx) de wikik — scénario de vérification et ticket de validation par l'utilisateur ou par le modèle. Utilisé quand le LLM ou l'agent produit, joue ou valide un test.
---

# Structure d'un test

Un fichier par test : `docs/projet/tests/TEST-xxx.md`. Chaque exigence a au moins un test.

## Frontmatter

`id`, `type: test`, `titre`, `statut`, `version`, `exigences: [EF-xxx]`, `validation` (`a_valider`, `utilisateur`, `modele`, `echec`), `valide_le` (date), `valide_par`.

## Corps

- **Préalable** : l'état de départ.
- **Étapes** : ce qu'on fait, une action par ligne.
- **Attendu** : ce qu'on doit constater, vérifiable sans interprétation.
- **Automatisation** : le test de code qui le joue, s'il existe (`chemin::test`).

## Ticket de validation

Un test passe à `utilisateur` quand l'utilisateur l'a constaté, à `modele` quand le modèle l'a vérifié (test automatisé vert, contrôle à l'écran), à `echec` sinon, avec une phrase.
