---
id: TEST-021
type: test
titre: "Un clic remplace l'onglet d'aperçu, un double-clic le fixe"
statut: proposé
version: v9
exigences: [EF-169]
validation: modele
---

# TEST-021 : Un clic remplace l'onglet d'aperçu, un double-clic le fixe

**Préalable** : Projet ouvert, onglets vidés.

**Étapes** :

1. Cliquer un document du volet, puis un autre.
2. Double-cliquer l'onglet en italique.
3. Cliquer un troisième document.

**Attendu** : Un seul onglet en italique change de page ; après le double-clic il reste, et le troisième document ouvre un nouvel onglet d'aperçu. Fermer l'onglet actif affiche son voisin.

**Automatisation** : headless : localStorage des onglets suivi pas à pas (lots 1 et 2).
