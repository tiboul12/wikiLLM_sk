---
id: TEST-012
type: test
titre: "La cascade descend du besoin au ticket de test"
statut: proposé
version: v8
exigences: [EF-160]
validation: modele
valide_le: 2026-10-22
---

# TEST-012 : La cascade descend du besoin au ticket de test

**Préalable** : Un besoin, une exigence qui le couvre.

**Étapes** :

1. Ouvrir Projet › Besoins › Cascade.
2. « ＋ test » sur une exigence sans test.
3. Sur le test : ✓ Je valide.

**Attendu** : Le test TEST-xxx est créé avec exigences et version de l'exigence ; son ticket passe à « validé (toi) » ; check signale les exigences sans test des versions testées.

**Automatisation** : contrôle headless de /projet/cascade ; route /api/document/valider-test.
