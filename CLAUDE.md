# wikiK : consignes pour Claude Code

wikiK est en **refonte** : la v0 Python (wiki Markdown éditable) devient un binaire Rust qui indexe la doc écrite dans le code, détecte la doc périmée et répond aux requêtes de l'agent, puis affiche un wiki local.

## Où on en est

- **Plan de référence** : `docs/projet/plan-v1.md` (phases 0 à 9, tableau de suivi en section 7). Le lire avant toute tâche.
- **Ce qui est reporté après la v1** : `docs/projet/backlog.md`, section « Reporté après la v1 ».
- **Réflexions d'origine et analyse** : `.doc/reflexion et evolution/` (non versionné).
- **v0 Python** : retirée du dépôt, consultable au tag `v0-python` (ADR-0007).
- **Outil à utiliser sur ce dépôt** : `wikik` lui-même (skill `skill/SKILL.md`, lié dans `.claude/skills/wikik`). `make check` avant chaque commit ; `wikik check` doit rester à zéro erreur.

## Règles

1. **Pas de code de la cible avant la porte de la Phase 3** (BDR, cahier des charges, SPEC-01 à 04 et ADR 002 à 005 validés par Tibo). Exception : le spike jetable de la Phase 1, dans `spikes/` (ignoré par git).
2. **Doc dans le même commit que le code** : le bloc `///` ou `//!` suit la fonction qu'il décrit ; poser `@req EF-xxx` quand une exigence est implémentée, `@adr NNNN` quand le code applique une décision.
3. **Un bloc ne paraphrase pas le code** : il dit l'intention, le contrat, les pièges, le lien vers une décision.
4. **Une phase = une branche** `phase-N-nom`, fusionnée dans `master` quand le critère de sortie du plan est atteint. En fin de phase, mettre à jour le tableau de suivi du plan, et le backlog si quelque chose est reporté.
5. **Documents de `docs/projet/`** : frontmatter YAML (`id`, `type`, `statut`, `version`, `sources`), un fichier par identifiant (`B-xxx`, `EF-xxx`, `ENF-xxx`, ADR).
6. **Questions ouvertes** : proposer une recommandation argumentée, poser une question à la fois. Échanges en français, messages de taille moyenne, avis tranché.

## Décisions de cadrage (détail : plan, section 1)

A1 solo et local, architecture prête pour l'équipe · A2 cœur Rust, un seul binaire · A3 front TypeScript, WASM réservé au site statique · A4 terrains d'essai : l'outil lui-même, puis ATS, puis shelley · A5 index et requêtes agent d'abord, wiki ensuite · A6 processus allégé, specs juste avant leur phase.
