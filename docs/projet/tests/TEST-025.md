---
id: TEST-025
type: test
titre: "La recherche par le sens répond avec un vrai modèle"
statut: proposé
version: v9
exigences: [EF-173]
validation: a_valider
---

# TEST-025 : La recherche par le sens répond avec un vrai modèle

**Préalable** : Ollama joignable, `ollama pull nomic-embed-text` fait.

**Étapes** :

1. Recherche › taper « comment zoomer dans le graphe » › ≈ Par le sens.

**Attendu** : Les premiers résultats parlent du zoom du graphe (PageGraphe, molette, REC-002).

**Automatisation** : circuit vérifié contre un faux Ollama ; à valider avec le vrai modèle.
