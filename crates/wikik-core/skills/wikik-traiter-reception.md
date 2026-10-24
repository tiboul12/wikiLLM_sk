---
name: wikik-traiter-reception
description: Traite les remarques ouvertes du cahier de réception du projet (outils MCP reception et reception_traiter de wikik) — plan par lots validé par l'utilisateur, puis correction, vérification, commit et enregistrement de chaque remarque. À utiliser quand l'utilisateur demande de traiter, corriger ou solder les remarques de réception.
---

# Traiter le cahier de réception

## Principes

- Ne jamais coder avant que l'utilisateur ait validé le plan.
- Une remarque n'est enregistrée comme traitée qu'une fois corrigée, vérifiée et commitée.
- Travailler lot par lot, pas remarque par remarque en vrac et pas tout d'un coup.
- Répondre en français.

## Étape 1 : lire les remarques ouvertes

1. Outil MCP `reception` (ou `wikik query reception`) : les remarques ouvertes, les bloquantes d'abord, avec le code de la page (`chemin::Symbole`), ses exigences et les chemins des captures.
2. Lire chaque remarque en entier, captures comprises (les ouvrir avec l'outil de lecture d'images).
3. Partir du symbole donné (`symbole`, `impact`) : pas de recherche à l'aveugle.
4. S'il n'y a aucune remarque ouverte, le dire et s'arrêter.

## Étape 2 : présenter le plan et attendre la validation

- Regrouper les remarques par lot cohérent (même écran, même composant, même type de correction).
- Donner l'ordre des lots et sa raison (dépendances, risque, rapidité).
- Pour chaque remarque : son identifiant, ce qui sera changé (fichiers, approche) et, si elle est ambiguë, la question à trancher.
- Signaler celles qu'il vaut mieux ne pas corriger (hors périmètre, déjà résolues) avec la raison.

Puis **s'arrêter et attendre la validation explicite** de l'utilisateur.

## Étape 3 : exécuter lot par lot

1. **Corriger** sans déborder du périmètre validé ; la doc (`///`, `@req`) change dans le même commit que le code.
2. **Vérifier** : tests, build, `wikik check` ; contrôler que le comportement décrit par la remarque est corrigé.
3. **Commiter** : un commit par lot, qui cite les remarques.
4. **Enregistrer** chaque remarque avec `reception_traiter` : statut (`traitee`, `reportee`, `rejetee`), une phrase qui dit ce qui a été fait ou pourquoi rien ne l'a été, hash du commit.

Une remarque qui demande une décision produit reste ouverte : la signaler à l'utilisateur.

## Étape 4 : conclure

Quelques lignes : remarques traitées, non traitées avec leur raison, commits créés.

## Garde-fous

- Ne jamais modifier `docs/reception/` à la main : c'est le cahier qui tient ces fichiers.
- Ne pas clore une remarque non vérifiée ou non commitée.
