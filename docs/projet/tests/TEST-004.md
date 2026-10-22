---
id: TEST-004
type: test
titre: "Le graphe est gardé, rechargeable, cherchable et relie fichiers et pages"
statut: proposé
version: v8
exigences: [EF-153]
validation: modele
valide_le: 2026-10-22
---

# TEST-004 : Le graphe est gardé, rechargeable, cherchable et relie fichiers et pages

**Préalable** : wikik lancé sur wikiK.

**Étapes** :

1. Attendre quelques secondes sur l'accueil.
2. Ouvrir le graphe.
3. Chercher EF-153.
4. Presser ↻.

**Attendu** : Le graphe s'ouvre en moins d'une seconde ; EF-153 est mis en avant et zoomé ; les arêtes appel/rend et navigue/contient existent (/api/graphe).

**Automatisation** : script headless graphe.mjs : 832 ms après préchargement.
