---
id: TEST-011
type: test
titre: "Ctrl+Z et Ctrl+Y annulent et rétablissent"
statut: proposé
version: v8
exigences: [EF-159]
validation: modele
valide_le: 2026-10-22
---

# TEST-011 : Ctrl+Z et Ctrl+Y annulent et rétablissent

**Préalable** : Une écriture faite depuis wikik.

**Étapes** :

1. Valider un document.
2. Ctrl+Z hors d'un champ, puis Ctrl+Y.

**Attendu** : Le fichier revient à l'état d'avant puis est réécrit ; un fichier modifié entre-temps n'est pas écrasé.

**Automatisation** : crates/wikik-core/src/journal.rs::tests::annuler_et_retablir et annuler.mjs.
