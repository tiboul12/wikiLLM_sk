---
id: TEST-030
type: test
titre: "Une fiche corrigée ne vaut que pour son atelier"
statut: proposé
version: v9
exigences: [EF-178]
validation: modele
---

# TEST-030 : Une fiche corrigée ne vaut que pour son atelier

**Préalable** : Deux ateliers.

**Étapes** :

1. Dans l'atelier n° 2, ouvrir ⚙ Fiche de la technique du Pitch, modifier une consigne, enregistrer.
2. Ouvrir le Pitch de l'atelier n° 1.

**Attendu** : L'atelier n° 2 montre « corrigée pour cet atelier », l'atelier n° 1 garde le texte d'origine ; une fiche qui change d'identifiant est refusée.

**Automatisation** : test d'intégration fiche_corrigee_par_atelier.
