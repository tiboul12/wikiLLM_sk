---
id: TEST-035
type: test
titre: "Git explique ce qu'il montre"
statut: proposé
version: v9
exigences: [EF-183]
validation: modele
---

# TEST-035 : Git explique ce qu'il montre

**Préalable** : Dépôt avec des fichiers non commités.

**Étapes** :

1. Git › cliquer le compteur de fichiers.
2. Lire le tableau des branches.
3. Git › Publication.

**Attendu** : Les fichiers sont des liens groupés par dossier ; chaque branche dit d'où elle part et si elle est fusionnée ; la page Publication coche les étapes et montre le skill.

**Automatisation** : headless (lot 8).
