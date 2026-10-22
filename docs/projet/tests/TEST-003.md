---
id: TEST-003
type: test
titre: "La recherche a son champ et une recherche par IA"
statut: proposé
version: v8
exigences: [EF-152]
validation: modele
valide_le: 2026-10-22
---

# TEST-003 : La recherche a son champ et une recherche par IA

**Préalable** : wikik lancé avec un LLM.

**Étapes** :

1. Ouvrir /recherche.
2. Taper une question, « ✦ Demander à l'IA ».

**Attendu** : Les résultats plein texte, puis les éléments rendus par le LLM, chacun lié avec sa raison.

**Automatisation** : route /api/recherche/ia appelée, page contrôlée en headless.
