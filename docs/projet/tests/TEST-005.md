---
id: TEST-005
type: test
titre: "La page fichier ne recalcule pas le check"
statut: proposé
version: v8
exigences: [EF-154]
validation: modele
valide_le: 2026-10-22
---

# TEST-005 : La page fichier ne recalcule pas le check

**Préalable** : wikik lancé sur wikiK.

**Étapes** :

1. Ouvrir deux pages de fichier de suite.

**Attendu** : /api/check répond en moins de 50 ms après le premier calcul (fait au démarrage).

**Automatisation** : mesure curl : 2,4 s → 24 ms.
