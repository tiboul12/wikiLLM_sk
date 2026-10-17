# Créer un ADR

1. Numéro : le plus grand numéro de `docs/projet/adr/` + 1, sur 4 chiffres (`0007`). En cas de conflit à la fusion, renuméroter le plus récent.
2. Copier le gabarit `docs/modeles/adr.md` (créé par `wikik init`) vers `docs/projet/adr/NNNN-titre-court.md`.
3. Frontmatter : `id: ADR-NNNN`, `statut: proposé`, `date`, `remplace` (ADR remplacés), `exigences` (EF/ENF concernées).
4. Corps : Contexte, Décision, Conséquences, Alternatives écartées. Une page maximum.
5. Poser `@adr N` dans le bloc de chaque symbole qui applique la décision.
6. Si l'ADR en remplace un autre : passer l'ancien en `statut: remplacé` et ajouter `remplace_par: [ADR-NNNN]`.
7. `wikik check` (zéro `ref-adr`), puis `wikik query adr N` pour vérifier les liens.
