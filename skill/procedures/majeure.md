# Ouvrir une mise à jour majeure

Rien de technique (ADR, spec, code) ne commence avant que la BDR puis le cahier des charges soient validés.

Avec l'atelier (section Atelier de `wikik`, voir `atelier.md`), les étapes 2 à 4 sont produites par le parcours (bilan, exigences, roadmap) en statut `proposé` : il reste à les relire et à les faire valider.

1. Créer `docs/projet/majeures/vN/` à partir des gabarits de `docs/modeles/` (`wikik init`) : `bdr-1-analyse.md`, `bdr-2-definition.md`, `cahier-des-charges.md`.
2. **BDR 1, analyse** : contexte, personnes concernées, irritants **avec preuves** (tickets, mesures, constats), contraintes, hors sujet. Aucune solution technique. Faire valider.
3. **BDR 2, définition** : objectifs mesurables, périmètre, cas d'usage, besoins. Chaque besoin a son fichier `docs/projet/besoins/B-xxx.md` (gabarit `besoin.md`). Faire valider (`statut: validé`).
4. **Cahier des charges** : une exigence par fichier `docs/projet/exigences/EF-xxx.md` ou `ENF-xxx.md` (gabarit `exigence.md`), avec énoncé, critère d'acceptation et `besoins:`. Les identifiants ne sont jamais réutilisés. La page `cahier-des-charges.md` n'est qu'une liste de liens.
5. `wikik check` : zéro `lien-casse`. Faire valider.
6. Ensuite seulement : specs (`docs/projet/specs/`), ADR, code avec `@req`.

Une mise à jour mineure ne refait pas de BDR : elle cite une exigence existante.
