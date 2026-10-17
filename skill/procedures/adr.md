# Créer un ADR

1. `wikik nouveau adr "Titre court"` crée `docs/projet/adr/NNNN-titre-court.md` au prochain numéro libre. En cas de conflit à la fusion (`id-duplique`), renuméroter le plus récent.
2. Compléter le fichier créé (gabarit déjà appliqué).
3. Frontmatter : `id: ADR-NNNN`, `statut: proposé`, `date`, `remplace` (ADR remplacés), `exigences` (EF/ENF concernées).
4. Corps : Contexte, Décision, Conséquences, Alternatives écartées. Une page maximum.
5. Poser `@adr N` dans le bloc de chaque symbole qui applique la décision.
6. Si l'ADR en remplace un autre : passer l'ancien en `statut: remplacé` et ajouter `remplace_par: [ADR-NNNN]`.
7. `wikik check` (zéro `ref-adr`), puis `wikik query adr N` pour vérifier les liens.
