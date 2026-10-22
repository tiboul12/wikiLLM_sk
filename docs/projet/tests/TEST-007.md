---
id: TEST-007
type: test
titre: "Le LLM se connecte à un modèle local (Ollama, llama.cpp)"
statut: proposé
version: v8
exigences: [EF-155]
validation: a_valider
---

# TEST-007 : Le LLM se connecte à un modèle local (Ollama, llama.cpp)

**Préalable** : Un serveur Ollama ou llama.cpp joignable (Shelley).

**Étapes** :

1. Paramètres › Connexion : Ollama, adresse du serveur.
2. Choisir un modèle de la liste, « Tester la connexion ».

**Attendu** : La liste des modèles vient de /api/tags (ou /v1/models) et le test répond.

**Automatisation** : aucun serveur local joignable depuis la machine de développement.
