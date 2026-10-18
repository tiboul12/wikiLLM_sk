---
id: BDR-v5-1
type: bdr-analyse
titre: "Analyse du besoin v5 (cahier de réception)"
statut: proposé
version: v5
sources: [MESURE-03]
---

# BDR v5, partie 1 : analyse du besoin

> Aucune solution technique dans ce document. Matière : la demande de Tibo du 18/10/2026, à la première utilisation de l'atelier.

## Contexte

Une application livrée passe par une phase de réception : son utilisateur la prend en main et a beaucoup de remarques à faire (bugs, ergonomie, évolutions, questions). Tibo commence celle de l'atelier v4, et en fera autant pour ses autres projets (ATS, shelley, meteoKry).

## Personnes concernées

Tibo, utilisateur qui reçoit l'application ; l'agent de code, qui applique ensuite les évolutions.

## Situation actuelle et irritants

| Irritant | Preuve |
|---|---|
| Les remarques arrivent en vrac, au fil des messages, mêlées aux questions. | Les retours sur l'atelier (« usine à gaz », adresse IP, plusieurs projets) sont venus un par un dans la conversation, chacun traité à chaud. |
| Une capture d'écran se perd dans la conversation. | Pas de lieu où ranger une image avec le texte qui l'explique. |
| Rien ne dit quelles pages ont été passées en revue. | Aucune liste des écrans de l'application. |
| La liste des écrans serait à recopier pour chaque projet. | Chaque projet déclare ses écrans autrement : routes Angular (shelley), `enum Page` Leptos (ATS), composants dans `pages/` (wikik), pages HTML (shelley-accueil). |
| Au moment de corriger, l'agent doit retrouver le code de l'écran. | La remarque désigne l'écran par son nom, pas par son code. |

## Contraintes

- Un seul outil (binaire wikik), local, même modèle que l'atelier : des fichiers dans le dépôt.
- Transposable à tout projet sans configuration ; réutiliser l'index wikik quand il existe.

## Hors sujet

- Les tests automatisés et la recette formelle (procès-verbal signé) : c'est le ressenti d'un utilisateur.
- Les remarques de plusieurs personnes en même temps (solo).
