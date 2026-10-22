---
id: TEST-002
type: test
titre: "Les pages s'ouvrent en onglets détachables"
statut: proposé
version: v8
exigences: [EF-151]
validation: modele
valide_le: 2026-10-22
---

# TEST-002 : Les pages s'ouvrent en onglets détachables

**Préalable** : wikik lancé, au moins deux documents.

**Étapes** :

1. Ouvrir « À relire ».
2. Ctrl+clic sur un document, clic du milieu sur un autre.
3. Recharger la page.
4. Cliquer ↗ sur un onglet.

**Attendu** : Trois onglets ; ils survivent au rechargement ; ↗ ouvre l'onglet dans une fenêtre sans barre d'onglets et le ferme ici.

**Automatisation** : script headless onglets.mjs du 22/10/2026.
