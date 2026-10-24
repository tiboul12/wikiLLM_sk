---
id: TEST-034
type: test
titre: "La carte de réception n'a pas de fausse page"
statut: proposé
version: v9
exigences: [EF-182]
validation: modele
---

# TEST-034 : La carte de réception n'a pas de fausse page

**Préalable** : Scanner à nouveau.

**Étapes** :

1. Réception › Scanner à nouveau.

**Attendu** : Plus de page « X » ni « Vrai » : les exemples des commentaires ne sont pas lus.

**Automatisation** : test unitaire exemples_des_commentaires_ignores et rescan.
