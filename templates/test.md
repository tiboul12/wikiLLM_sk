---
id: TEST-000           # TEST-xxx ; au moins un par exigence
type: test
titre: Ce que le test vérifie
statut: proposé        # proposé | validé
version: v1
exigences: [EF-000]
validation: a_valider  # a_valider | utilisateur | modele | echec
---

# TEST-000 : Ce que le test vérifie

**Préalable** : l'état de départ.

**Étapes** :

1. ce qu'on fait, une action par ligne.

**Attendu** : ce qu'on doit constater, sans interprétation.

**Automatisation** : le test de code qui le joue, s'il existe (`chemin::test`).
