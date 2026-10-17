---
id: ADR-0010
type: adr
titre: "Atelier : données en fichiers du dépôt"
statut: accepté
date: 2026-10-17
remplace: []
exigences: []
---

# ADR-0010 : Atelier : données en fichiers du dépôt

## Contexte

Les productions de l'atelier (pitch, idées, analyses, storyboards, besoins, exigences, roadmap) doivent être lues par le wiki et par l'agent de code, et garder leur historique (décision B3).

## Décision

- Chaque artefact de l'atelier est **un fichier Markdown** avec frontmatter YAML, sous **`docs/atelier/`** : un fichier par identifiant (`PITCH`, `IDEE-xxx`, `ANA-xxx`, `SCA-xxx`, `SB-xxx`…), même règle que la zone Projet (A1).
- Ce qui sort de l'atelier pour le développement (besoins `B-xxx`, BDR, exigences, roadmap) est écrit dans **`docs/projet/`** avec les gabarits existants.
- La **traçabilité** passe par le champ `sources` du frontmatter : chaque artefact cite ceux dont il vient.
- L'index lit `docs/atelier/` comme une deuxième zone de documents.

## Conséquences

- Aucune base propre à l'atelier : git est la mémoire du projet, et l'historique d'un besoin se lit dans `git log`.
- Les brouillons sont versionnés ; ils se distinguent du validé par leur dossier et leur statut.

## Alternatives écartées

- **Base SQLite de l'atelier** : variantes et votes plus simples à stocker, mais il faudrait une passerelle vers le wiki et on perdrait l'historique git.
