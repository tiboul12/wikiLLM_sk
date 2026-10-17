---
id: BDR-v3-1
type: bdr-analyse
titre: "Analyse du besoin v3 (équipe)"
statut: proposé
version: v3
sources: [BDR-v1-2, BACKLOG]
---

# BDR v3, partie 1 : analyse du besoin

> Aucune solution technique dans ce document.

## Contexte

La v1 a été pensée pour passer en équipe sans réécriture (décision A1) : écriture uniquement par git, index déterministe, un fichier par identifiant. Ce qui manque, c'est l'outillage du travail à plusieurs : relire, publier, valider.

## Personnes concernées

L'équipe qui travaillera avec Tibo sur ses dépôts, leurs agents, et les relecteurs des documents produit (BDR, cahier des charges).

## Situation actuelle et irritants

| Irritant | Preuve |
|---|---|
| Le wiki n'existe que sur la machine qui lance `wikik serve`. | `serve` écoute sur `127.0.0.1` par conception (ENF-004). |
| Rien n'empêche de fusionner une PR qui rend la doc fausse. | `check` ne tourne qu'à la main. |
| Deux personnes peuvent créer le même `EF-xxx` en parallèle. | Les identifiants sont choisis à la main. |
| Personne n'est désigné pour valider une BDR ou un cahier des charges. | Question Q10 de la roadmap, toujours ouverte. |
| On ne voit pas l'évolution de la couverture de doc. | Les chiffres n'existent que dans l'accueil du wiki local. |

## Contraintes

- Rien d'hébergé par wikik : la publication passe par l'hébergement statique du dépôt (GitHub Pages ou équivalent).
- Le site publié ne doit jamais permettre d'écrire.

## Hors sujet

- Backlog dans un outil de tickets (Q1), portail multi-dépôts (Q7) : décisions laissées à Tibo.
- Comptes, droits fins, édition en ligne.
