---
id: TEST-017
type: test
titre: "Une remarque traitée se valide et montre son état git"
statut: proposé
version: v8
exigences: [EF-165]
validation: modele
valide_le: 2026-10-22
---

# TEST-017 : Une remarque traitée se valide et montre son état git

**Préalable** : Une remarque traitée avec un commit.

**Étapes** :

1. Traitement › À valider : ✓ Je valide ; sur une autre, ↺ À reprendre avec un commentaire.

**Attendu** : La première est validée ; la seconde redevient ouverte avec le commentaire dans son texte ; chaque correction montre commit, poussé ou non, version, avec sa couleur.

**Automatisation** : contrôle headless de /reception, /reception/traitement et de la carte d'une page.
