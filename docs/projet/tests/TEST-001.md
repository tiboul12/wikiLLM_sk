---
id: TEST-001
type: test
titre: "Les outils sont en bas et les accordéons fermés à l'arrivée"
statut: proposé
version: v8
exigences: [EF-150]
validation: modele
valide_le: 2026-10-22
---

# TEST-001 : Les outils sont en bas et les accordéons fermés à l'arrivée

**Préalable** : wikik lancé sur wikiK.

**Étapes** :

1. Ouvrir l'accueil.
2. Regarder la barre d'activité.
3. Ouvrir la section Projet, puis presser ⊞ deux fois.

**Attendu** : Code (avec le lien Graphe en tête de son panneau) et Git sont en bas, au-dessus des Paramètres ; les accordéons du volet Projet sont fermés à l'arrivée ; ⊞ déplie tout, ⊟ replie tout ; « À relire » est dans Gestion › À traiter.

**Automatisation** : contrôle headless (Playwright) du 22/10/2026.
