---
id: TEST-014
type: test
titre: "La clarification juge la question et lie ce qu'elle cite"
statut: proposé
version: v8
exigences: [EF-162]
validation: a_valider
---

# TEST-014 : La clarification juge la question et lie ce qu'elle cite

**Préalable** : Une clarification dont la question est hors sujet.

**Étapes** :

1. Ouvrir la clarification, laisser le LLM proposer.

**Attendu** : Un bandeau « Question hors sujet » s'affiche si le LLM le juge ; les identifiants, fichiers et routes d'écran des pistes sont des liens.

**Automatisation** : à constater avec un vrai LLM (le fournisseur factice ne juge pas).
