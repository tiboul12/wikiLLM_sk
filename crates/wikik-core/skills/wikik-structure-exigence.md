---
name: wikik-structure-exigence
description: Comment rédiger une exigence fonctionnelle (EF-xxx) ou non fonctionnelle (ENF-xxx) de wikik — énoncé vérifiable, critère d'acceptation, besoins couverts, tests. Utilisé quand le LLM ou l'agent produit ou corrige une exigence.
---

# Structure d'une exigence

Un fichier par exigence : `docs/projet/exigences/EF-xxx.md` (ou `ENF-xxx`), portée par le cahier des charges de sa version.

## Frontmatter

`id`, `type: exigence` (ou `exigence-nf`), `titre`, `statut`, `priorite` (`must`, `should`, `could`), `version`, `besoins: [B-xxx]`.

## Corps

- **Énoncé** : ce que le produit fait, vérifiable, sans dire comment il est codé.
- **Critère d'acceptation** : comment on sait que c'est fait (mesure, remarque traitée, scénario).

## Cascade

Besoin → cahier des charges → exigence → **un test par exigence** (`TEST-xxx`, champ `exigences`) → ticket de validation (utilisateur ou modèle). Le code qui l'implémente porte `@req EF-xxx`.
