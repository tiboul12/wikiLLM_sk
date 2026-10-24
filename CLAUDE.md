# wikiK : consignes pour Claude Code

wikiK est en **refonte** : la v0 Python (wiki Markdown éditable) devient un binaire Rust qui indexe la doc écrite dans le code, détecte la doc périmée et répond aux requêtes de l'agent, puis affiche un wiki local.

## Où on en est

- **Versions** (tags poussés sur GitHub le 22/10/2026) : `v1.0-rc1` (index, check, requêtes, wiki), `v1.1` serveur MCP, `v2.0-rc1` TypeScript, `v2.1` JavaScript et Svelte, `v3.0-rc1` équipe (export statique, CI, annotations, `nouveau`, `stats`). Tout a été produit par délégation le 17/10/2026 : BDR, specs et ADR attendent la relecture de Tibo.
- **v4.0-rc1** (18/10/2026) : atelier de définition du besoin (Fast Design Thinking, 12 fiches dans `atelier/techniques/`), LLM `claude -p` branché sur le MCP de wikik, production des besoins, BDR, exigences et roadmap, rappel `atelier_rappel`. Plan : `docs/projet/majeures/v4/plan-v4.md` ; mesure : MESURE-03.
- **v5.0-rc1** (18/10/2026) : cahier de réception (carte des pages construite à partir du code, remarques avec captures, traitement, outils MCP `pages`, `reception`, `reception_traiter`). Plan : `docs/projet/majeures/v5/plan-v5.md`.
- **v6.0-rc1** (19/10/2026) : un seul serveur et une seule commande, `wikik` (ADR-0012) : accueil, barre d'activité à gauche (Produit, Projet, Atelier, Réception), accueil par section. `serve`, `atelier`, `reception` n'existent plus.
- **v7** (19/10/2026, branche `v7-retours`, pas encore fusionnée) : traitement des 40 remarques de la réception de la v6 (PLAN-v7) : noms et navigation, graphe par catégorie, récapitulatifs, gestion en colonnes, clarifications par sujet, parcours multiples, LLM annulable, Paramètres et terminal intégré (ADR-0013).
- **v8** (22/10/2026, branche `v8-retours`, partie de `v7-retours`, pas encore fusionnée) : traitement des 41 remarques de la réception de la v7 (PLAN-v8) : outils en bas (Code, Git), onglets détachables, graphe gardé et cherchable, connecteurs LLM (ADR-0015), consignes en skills (ADR-0017), édition des documents et Ctrl+Z (ADR-0016), cascade jusqu'aux tests (`TEST-xxx`), ateliers en sessions, validation des remarques et état git, historique git, dessin (Mermaid, SVG). REC-009.2 (front en Rust / WebAssembly) passe en **v9**, en attente du design system Rust que Tibo fournira.
- **Prochaine étape avec Tibo** : recevoir la v8 (backlog, section « v8 »), puis `docs/projet/backlog.md`, sections « À faire avec Tibo » (v1 et v4 : lancer le projet « météo » dans l'atelier, rappel sur ATS).

- **Plan de référence** : `docs/projet/plan-v1.md` (phases 0 à 9, tableau de suivi en section 7). Le lire avant toute tâche.
- **Ce qui est reporté après la v1** : `docs/projet/backlog.md`, section « Reporté après la v1 ».
- **Réflexions d'origine et analyse** : `.doc/reflexion et evolution/` (non versionné).
- **v0 Python** : retirée du dépôt, consultable au tag `v0-python` (ADR-0007).
- **Outil à utiliser sur ce dépôt** : `wikik` lui-même (skill `skill/SKILL.md`, lié dans `.claude/skills/wikik`). `make check` avant chaque commit ; `wikik check` doit rester à zéro erreur et `wikik stats` à 95 % de symboles documentés au moins (100 % le 22/10/2026).

## Règles

1. **Toute version majeure commence par sa BDR et son cahier des charges** (`wikik nouveau`, gabarits dans `templates/`), puis specs et ADR, puis le code. Les mineures citent une exigence existante.
2. **Doc dans le même commit que le code** : le bloc `///` ou `//!` suit la fonction qu'il décrit ; poser `@req EF-xxx` quand une exigence est implémentée, `@adr NNNN` quand le code applique une décision.
3. **Un bloc ne paraphrase pas le code** : il dit l'intention, le contrat, les pièges, le lien vers une décision.
4. **Une phase = une branche** `phase-N-nom`, fusionnée dans `master` quand le critère de sortie du plan est atteint. En fin de phase, mettre à jour le tableau de suivi du plan, et le backlog si quelque chose est reporté.
5. **Documents de `docs/projet/`** : frontmatter YAML (`id`, `type`, `statut`, `version`, `sources`), un fichier par identifiant (`B-xxx`, `EF-xxx`, `ENF-xxx`, ADR).
6. **Questions ouvertes** : proposer une recommandation argumentée, poser une question à la fois. Échanges en français, messages de taille moyenne, avis tranché.

## Décisions de cadrage (détail : plan, section 1)

A1 solo et local, architecture prête pour l'équipe · A2 cœur Rust, un seul binaire · A3 front TypeScript, WASM réservé au site statique · A4 terrains d'essai : l'outil lui-même, puis ATS, puis shelley · A5 index et requêtes agent d'abord, wiki ensuite · A6 processus allégé, specs juste avant leur phase.
