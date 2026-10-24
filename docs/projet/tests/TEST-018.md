---
id: TEST-018
type: test
titre: "L'historique git se lit en graphe de branches"
statut: proposé
version: v8
exigences: [EF-166]
validation: modele
valide_le: 2026-10-22
---

# TEST-018 : L'historique git se lit en graphe de branches

**Préalable** : Un dépôt avec branches et fusions.

**Étapes** :

1. Ouvrir Git.
2. Cliquer un commit, puis un fichier.

**Attendu** : Un couloir par branche, fusions dessinées ; refs sur leurs commits ; message, fichiers et diff coloré.

**Automatisation** : dépôt d'essai à deux branches fusionnées, graphe contrôlé en headless.
