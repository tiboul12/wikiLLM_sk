---
id: TEST-024
type: test
titre: "La recherche floue trouve malgré la casse et une faute"
statut: proposé
version: v9
exigences: [EF-172]
validation: modele
---

# TEST-024 : La recherche floue trouve malgré la casse et une faute

**Préalable** : Page Recherche et liste des clarifications.

**Étapes** :

1. Taper « ef-14 », puis « onglets aprecu ».

**Attendu** : Les suggestions contiennent EF-140… puis EF-169 malgré l'inversion de lettres.

**Automatisation** : headless (lots 7 et 9).
