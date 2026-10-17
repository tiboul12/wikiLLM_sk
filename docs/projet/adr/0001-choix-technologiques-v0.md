---
id: ADR-0001
type: adr
titre: "Choix technologiques de la v0 (Markdown, Python standard, SPA)"
statut: remplacé
date: 2026-09-21
remplace: []
remplace_par: [ADR-0002]
exigences: []
---

# ADR-0001 : Choix technologiques de la v0

> Repris de `wiki/decisions/001-choix-technologiques.md` (21/09/2026). **Remplacé par [[adr/0002-rust-binaire-unique]]** le 17/10/2026.

## Contexte

Il fallait une solution ultra-légère pour documenter les sessions de travail et les décisions avec un agent LLM, consultable et éditable depuis un navigateur, sans dépendance lourde ni base de données.

## Décision

1. Fichiers Markdown purs dans `wiki/`.
2. Serveur local en Python standard (`serveur.py`), sans dépendance pip.
3. Interface monopage (`index.html`) : rendu Markdown, éditeur avec sauvegarde, graphe des liens `[[...]]`.

## Conséquences

- Déploiement immédiat dans n'importe quel dépôt git ; compatible Obsidian et tout éditeur.
- Le wiki ne lit pas le code ; l'éditeur écrit hors du flux git ; pas d'authentification.
