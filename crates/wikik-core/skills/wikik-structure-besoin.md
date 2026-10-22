---
name: wikik-structure-besoin
description: Comment rédiger un besoin (B-xxx) de la zone Projet de wikik — qui, besoin, pourquoi, sans solution technique. Utilisé quand le LLM ou l'agent produit ou corrige un besoin.
---

# Structure d'un besoin

Un fichier par besoin : `docs/projet/besoins/B-xxx.md`.

## Frontmatter

`id`, `type: besoin`, `titre` (une phrase à l'infinitif, du point de vue de l'utilisateur), `statut` (`proposé` puis `validé`), `version`, `sources` (BDR, remarques, artefacts de l'atelier).

## Corps

- **Qui** : la personne concernée et sa situation.
- **Besoin** : ce qu'elle doit pouvoir faire, en une ou deux phrases.
- **Pourquoi** : l'irritant ou la preuve (remarque, mesure, verbatim).

## Règles

- Aucune solution technique : ni écran, ni technologie.
- Un besoin = un résultat attendu ; le découper s'il en porte deux.
- Seul ce qui demande une décision de l'utilisateur part en relecture.
