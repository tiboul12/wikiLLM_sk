---
id: TEST-010
type: test
titre: "Un document à relire se valide et s'édite"
statut: proposé
version: v8
exigences: [EF-158]
validation: modele
valide_le: 2026-10-22
---

# TEST-010 : Un document à relire se valide et s'édite

**Préalable** : Un besoin au statut proposé.

**Étapes** :

1. À relire › ✓ Valider.
2. Corriger : modifier le texte, Ctrl+S.

**Attendu** : Le statut passe à validé (accepté pour un ADR) ; le fichier est réécrit ; la liste ne contient que les types qui demandent une décision.

**Automatisation** : script headless annuler.mjs (ADR-0015 validé puis annulé).
