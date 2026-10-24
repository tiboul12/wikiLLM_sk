---
id: TEST-009
type: test
titre: "Ce qui attend une décision remonte jusqu'à Gestion"
statut: proposé
version: v8
exigences: [EF-157]
validation: modele
valide_le: 2026-10-22
---

# TEST-009 : Ce qui attend une décision remonte jusqu'à Gestion

**Préalable** : Backlog avec des cases dans une section ✋.

**Étapes** :

1. Ouvrir le volet Projet.
2. Mettre une page en favori (☆ de son onglet).

**Attendu** : Gestion affiche le total en orange ; À traiter détaille tâches du backlog, documents à relire, clarifications, remarques à valider ; la page apparaît dans Favoris.

**Automatisation** : contrôle headless du volet Projet.
