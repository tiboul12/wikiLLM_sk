---
id: SPEC-11
type: spec
titre: "Le skill wikik pour l'agent de code"
statut: proposé
version: v1
exigences: [EF-030, EF-031, EF-032]
decisions: [ADR-0005]
---

# SPEC-11 : Le skill wikik

## Problème

L'outil ne sert que si l'agent l'utilise au bon moment : interroger l'index avant d'ouvrir du code, mettre la doc à jour dans le même commit que le code. Ces réflexes s'écrivent dans un skill.

## Exigences

1. **R1** `SKILL.md` court (moins de 80 lignes) : quand l'utiliser, règles d'or, commandes. Les procédures détaillées sont dans des fichiers chargés à la demande.
2. **R2** Procédures : interroger avant de modifier, écrire un bloc, amorcer un projet existant, créer un ADR, ouvrir une mise à jour majeure (version simple, sans atelier).
3. **R3** Le skill ne contredit pas SPEC-01 ; il y renvoie.
4. **R4** Il remplace, sur les projets qui l'adoptent, les skills `doc-as-code` et `wiki-dev-sync` (conventions reprises : docstrings natives, ADR ; abandonnées : bloc ADR complet dans le code, journal de sessions, `code-map.md` monolithique).

## Proposition

Dossier `skill/` du dépôt : `SKILL.md` + `procedures/*.md`. Installation : lien vers ce dossier dans `~/.claude/skills/wikik` (global) ou `.claude/skills/wikik` (par projet).

## Critères d'acceptation

Un agent qui suit le skill sur ATS lance `wikik query` avant d'ouvrir un fichier source, et le `check` du commit qu'il produit ne contient aucun `perime-*` nouveau.
