---
id: CADRAGE-v4
type: cadrage
titre: "Cadrage de la v4 : atelier de définition du besoin (Fast Design Thinking)"
statut: brouillon
version: v4
sources: [BACKLOG]
---

# Cadrage de la v4 : atelier de définition du besoin

> Décisions prises avec Tibo avant le plan de la v4. Une ligne par décision, dans l'ordre où elles sont prises.

| # | Date | Décision | Conséquence |
|---|---|---|---|
| B1 | 2026-10-18 | Le framework Fast Design Thinking peut être versionné et publié sur git. | Sa version réécrite vit dans le dépôt. |
| B2 | 2026-10-18 | **L'atelier est une application interactive, vivante, pierre angulaire du projet** : il sert au lancement (définir le besoin, puis produire la doc de dev, puis le code) et on peut y **revenir en cours de développement** pour éclaircir un point. Ce n'est pas une simple procédure conversationnelle. | L'atelier a sa propre interface et appelle un LLM. Il tourne comme un **serveur séparé, relié au wiki**, pour que le wiki reste en lecture seule (ENF-004). |
| B3 | 2026-10-18 | **Les données de l'atelier sont des fichiers Markdown du dépôt**, dans `docs/atelier/`, avec la même structure que la zone Projet (frontmatter, un fichier par identifiant). | L'index, le wiki et l'agent les lisent sans passerelle ; git garde l'historique de chaque besoin. L'atelier écrit des fichiers ; il ne tient pas de base à lui. |
| B4 | 2026-10-18 | **LLM appelé par le CLI local** (`claude -p`, abonnement, pas de clé d'API), derrière une couche fournisseur générique comme dans ATS (ADR-0009 d'ATS). **Le LLM est branché sur le serveur MCP de wikik** : il lit lui-même le contenu de l'atelier et du projet au lieu de recevoir tout le contexte dans le prompt. | Vérifié sur la machine : `claude -p` accepte `--mcp-config`, `--allowedTools` (outils `mcp__wikik__*` seulement) et `--json-schema` (réponse structurée, utile pour produire des artefacts au bon format). Le serveur MCP de wikik devra exposer les pages de l'atelier. |
| B5 | 2026-10-18 | **Terrains d'essai** : (1) un projet neuf de Tibo, une app de « météo » crypto / investissement, pour le lancement complet (pitch → atelier → BDR → cahier des charges → code) ; (2) ATS, pour le rappel de l'atelier en cours de développement sur une exigence existante. | Le projet météo n'est pas cadré à l'avance : son pitch sera la première séance de l'atelier, ce qui teste l'outil sur un vrai démarrage. |
| B6 | 2026-10-18 | **Deux modes, Express et Complet**, au choix à chaque étape (passage de l'un à l'autre possible). **Le LLM propose des variantes en lot, Tibo garde, corrige ou rejette à l'écran** ; les frameworks de support (Q.U.A.D., I.D.E.A., I.V.A., E.T.A.T., B.L.O.C., M.E.T.R.O.) sont remplacés par l'application (objets identifiés, versionnés, cliquables). **Étape 0 « Pitch »** ajoutée. | Express : 1 axe de free writing, 3 idées retenues, SCAMPER sur 1 sujet, 1 à 2 storyboards. |
| B7 | 2026-10-18 | **L'atelier va du pitch jusqu'au cahier des charges et à la roadmap** : brouillons (`docs/atelier/`), besoins `B-xxx` et BDR, exigences `EF`/`ENF` reliées aux besoins et aux storyboards, découpage en versions. **Specs, ADR et code restent à Claude Code** pendant le développement, qui rappelle l'atelier quand un besoin est flou. | Frontière : décisions produit dans l'atelier, décisions techniques dans le dev. |
