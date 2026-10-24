---
id: TEST-013
type: test
titre: "Les ateliers sont des sessions modifiables, les clarifications à part"
statut: proposé
version: v8
exigences: [EF-161]
validation: modele
valide_le: 2026-10-22
---

# TEST-013 : Les ateliers sont des sessions modifiables, les clarifications à part

**Préalable** : wikik lancé.

**Étapes** :

1. Design thinking : ✎ Modifier l'atelier (titre, statut).
2. Ouvrir Clarifications.

**Attendu** : L'atelier n° 1 change de titre et de statut ; les clarifications sont un tableau à part, avec Reprendre et Supprimer.

**Automatisation** : contrôle headless de /atelier et /clarifications.
