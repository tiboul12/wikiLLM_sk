---
id: ADR-0007
type: adr
titre: "Pas de journal de sessions ; le wiki v0 n'est migré que pour ses décisions et son backlog"
statut: accepté
date: 2026-10-17
remplace: []
exigences: [EF-050]
---

# ADR-0007 : Journal de sessions et migration du wiki v0

## Contexte

La v0 tenait un journal `wiki/sessions/` (une page par séance agent + humain), une cartographie `code-map.md` régénérée en entier, des guides d'utilisation de la v0 et un backlog. La question Q4 de la roadmap demandait s'il fallait garder le journal.

## Décision

- **Pas de journal de sessions** dans la zone Projet : les commits, leurs messages et les ADR tiennent ce rôle, et un journal par séance devient du bruit à plusieurs.
- **Migré** : l'ADR 001 (devenu [[adr/0001-choix-technologiques-v0]], statut remplacé) et le backlog (devenu [[backlog]]).
- **Non migré** (consultable au tag `v0-python`) : les sessions, `code-map.md` (remplacé par `wikik query carte`), les guides de la v0 et sa vue d'architecture (remplacée par [[architecture]]).

## Conséquences

- `wiki/` et `legacy/` disparaissent du dépôt ; `git checkout v0-python` les rend.
- Le skill wikik ne demande jamais de journal de séance.

## Alternatives écartées

- **Migrer tout `wiki/`** : la moitié des pages décrit un outil qui n'existe plus.
