---
id: TEST-019
type: test
titre: "On dessine la carte mentale et les vignettes"
statut: proposé
version: v8
exigences: [EF-167]
validation: modele
valide_le: 2026-10-22
---

# TEST-019 : On dessine la carte mentale et les vignettes

**Préalable** : Un storyboard avec des vignettes, une carte mentale.

**Étapes** :

1. Étape Storyboard : ✎ Dessiner une vignette (rectangle, flèche, texte), enregistrer.
2. Étape Carte mentale : modifier le Mermaid.

**Attendu** : Le SVG est écrit dans docs/atelier/dessins/ et affiché dans la vignette ; l'aperçu Mermaid suit la frappe et s'enregistre dans la carte.

**Automatisation** : script headless dessin.mjs sur un projet d'essai.
