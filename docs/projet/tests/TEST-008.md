---
id: TEST-008
type: test
titre: "Les prompts système sont des skills modifiables"
statut: proposé
version: v8
exigences: [EF-156]
validation: modele
valide_le: 2026-10-22
---

# TEST-008 : Les prompts système sont des skills modifiables

**Préalable** : wikik lancé.

**Étapes** :

1. Paramètres › Prompts système : ouvrir « Lire le projet avec wikik ».
2. Le modifier, enregistrer ; puis « Revenir au texte d'origine ».

**Attendu** : Le fichier .claude/skills/wikik-lecture/SKILL.md est écrit puis supprimé ; un texte sans frontmatter est refusé.

**Automatisation** : test d'intégration crates/wikik/tests/parametres.rs::parametres_et_cle.
