---
id: TEST-029
type: test
titre: "L'atelier s'ouvre sur un accueil et se règle en entier"
statut: proposé
version: v9
exigences: [EF-177]
validation: modele
---

# TEST-029 : L'atelier s'ouvre sur un accueil et se règle en entier

**Préalable** : Design thinking.

**Étapes** :

1. Cliquer Design thinking.
2. Ouvrir un atelier, passer en Express, décocher une technique, enregistrer.

**Attendu** : L'accueil liste les ateliers ; la technique décochée passe « sautée » ; ✎ Modifier l'atelier est visible sur téléphone.

**Automatisation** : test d'intégration mode_de_tout_l_atelier et headless (lot 7).
