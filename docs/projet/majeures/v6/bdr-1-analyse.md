---
id: BDR-v6-1
type: bdr-analyse
titre: "Analyse du besoin v6 (un seul serveur, accueil et sections)"
statut: proposé
version: v6
sources: [ADR-0009]
---

# BDR v6, partie 1 : analyse du besoin

> Aucune solution technique dans ce document. Matière : les retours de Tibo des 18 et 19/10/2026.

## Contexte

wikik a maintenant quatre usages pour un projet : le code (Produit), les documents (Projet), l'atelier de définition du besoin et le cahier de réception.

## Personnes concernées

Tibo, qui passe d'un usage à l'autre au fil du projet ; plus tard, l'équipe.

## Situation actuelle et irritants (avec preuves)

| Irritant | Preuve |
|---|---|
| Trois commandes et trois ports pour un même projet. | « je trouve ça un peu usine à gaz de lancer mes serveurs » ; « je suis surpris qu'on ait fait 3 serveurs ». |
| Pas de point d'entrée : chaque usage s'ouvre à part, sans vue d'ensemble. | « j'ai besoin d'une page d'accueil qui m'emmène sur les différentes sections ». |
| Pas de navigation permanente entre usages. | « une barre sur le côté gauche avec tous les raccourcis vers ces sections, à la VS Code ». |
| Des commandes anciennes resteraient à maintenir. | « je ne veux pas garder de commandes historiques qui plus tard poseront des problèmes ». |

## Contraintes

- Le site exporté (publication) reste en lecture seule.
- Mêmes protections d'écriture que l'atelier et la réception.

## Hors sujet

- Plusieurs projets dans un même serveur (un serveur par projet).
