---
id: BDR-v8-2
type: bdr-definition
titre: "Définition du besoin v8 (retours de la réception de la v7)"
statut: proposé
version: v8
sources: [BDR-v8-1]
---

# BDR v8, partie 2 : définition du besoin

## Objectifs mesurables

- Les 41 remarques sont traitées, chacune avec son commit, dans le cahier de réception.
- Tibo valide ou corrige un document à relire sans quitter wikik, et annule une erreur par Ctrl+Z.
- Tibo voit d'un coup d'œil ce qui attend sa décision, section par section.
- Tibo choisit son LLM (Claude, Antigravity, Mistral, local), son modèle et son effort ; les consignes des LLM sont des skills qu'il peut relire et modifier.

## Périmètre (inclus / exclu)

**Inclus** : coquille (outils, onglets, accordéons, recherche), graphe, page fichier, paramètres et skills, gestion et relecture avec édition, cascade jusqu'aux tests, ateliers et clarifications, carte de réception de l'atelier, validation de la réception et état git des corrections, historique git, outils de dessin.

**Exclu** : le front en Rust / WebAssembly (REC-009.2, v9).

## Cas d'usage prioritaires

1. Tibo ouvre wikik, voit sur le volet Projet « Backlog 6 » à sa charge, ouvre le backlog dans un onglet, coche une tâche, valide un besoin à relire dans un second onglet.
2. Il passe le LLM de Claude Opus à Sonnet en effort moyen, ou à un modèle Ollama de Shelley, sans relancer wikik.
3. En fin de réception, il ouvre la carte d'une page, voit que la remarque est traitée, commitée, poussée et livrée en v8, et la valide.

## Besoins

| ID | Besoin | Priorité |
|---|---|---|
| [[besoins/B-019]] | Travailler sur plusieurs pages à la fois et voir ce qui attend une décision | must |
| [[besoins/B-020]] | Relire, valider et corriger la documentation du projet, en cascade jusqu'aux tests | must |
| [[besoins/B-021]] | Choisir son LLM et gouverner ses consignes comme des skills | must |
| [[besoins/B-022]] | Animer des ateliers de design thinking en sessions, clarifications à part, avec des outils de dessin | should |
| [[besoins/B-023]] | Valider la réception et suivre chaque correction jusqu'à sa version | must |

## Critères de succès de la version

Les remarques sont au statut « traitée » ; `make check` et `wikik check` passent ; contrôle headless clair, sombre et téléphone sans erreur.

## Hypothèses et risques

- L'édition dans le navigateur écrit dans le dépôt : réservée au propriétaire (ADR-0013), annulable (ADR-0016).
- Un modèle local n'a pas d'accès MCP : le contexte du projet lui est donné dans la demande (ADR-0015).

## Décision de lancement

Accordée par Tibo le 22/10/2026 (« je viens de faire une réception, tu peux lancer les dév jusqu'à épuisement des tokens »).
