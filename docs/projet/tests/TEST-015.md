---
id: TEST-015
type: test
titre: "Le bilan ne montre que les besoins de l'atelier"
statut: proposé
version: v8
exigences: [EF-163]
validation: modele
valide_le: 2026-10-22
---

# TEST-015 : Le bilan ne montre que les besoins de l'atelier

**Préalable** : Des besoins écrits à la main dans docs/projet, atelier vide.

**Étapes** :

1. Ouvrir l'étape Bilan, sous-étape tri.

**Attendu** : « Aucun besoin gardé pour l'instant » : les besoins du projet n'y sont pas.

**Automatisation** : contrôle headless de /atelier/bilan.
