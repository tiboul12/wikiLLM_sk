---
id: TEST-036
type: test
titre: "Le graphe zoome progressivement et éclaire les liens"
statut: proposé
version: v9
exigences: [EF-184]
validation: modele
---

# TEST-036 : Le graphe zoome progressivement et éclaire les liens

**Préalable** : Graphe d'un document.

**Étapes** :

1. Trois crans de molette sur une zone.
2. Double-clic sur un nœud.
3. Appui long sur un nœud.

**Attendu** : Le zoom accélère puis s'affine ; le double-clic zoome sans ouvrir ; l'appui long grise le reste et liste les nœuds liés avec leur couleur.

**Automatisation** : headless (lot 6).
