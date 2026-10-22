---
id: TEST-006
type: test
titre: "Le LLM se connecte par Claude Code, Antigravity ou Mistral Vibe"
statut: proposé
version: v8
exigences: [EF-155]
validation: modele
valide_le: 2026-10-22
---

# TEST-006 : Le LLM se connecte par Claude Code, Antigravity ou Mistral Vibe

**Préalable** : CLI connectés sur la machine.

**Étapes** :

1. Paramètres › Connexion : choisir le connecteur, le modèle, l'effort.
2. « Tester la connexion ».

**Attendu** : Chaque connecteur répond (le modèle et l'effort sont passés au CLI).

**Automatisation** : tests réels du 22/10/2026 : claude haiku/low 5 s, agy gemini-3.8-flash-low 12 s, vibe 23 s.
