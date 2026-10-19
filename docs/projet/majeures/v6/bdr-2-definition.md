---
id: BDR-v6-2
type: bdr-definition
titre: "Définition du besoin v6 (un seul serveur, accueil et sections)"
statut: proposé
version: v6
sources: [BDR-v6-1]
---

# BDR v6, partie 2 : définition du besoin

## Objectifs mesurables

- Une commande, `wikik`, ouvre tout le projet sur une adresse.
- Toute section est à un clic, de partout.

## Périmètre (inclus / exclu)

**Inclus** : serveur unique, suppression de `serve`, `atelier`, `reception` ; page d'accueil ; barre d'activité à gauche ; page d'accueil de chaque section.

**Exclu** : changement des écrans internes de l'atelier et de la réception.

## Cas d'usage prioritaires

1. Tibo lance `wikik` dans un projet, voit l'état des quatre sections et ouvre celle qui l'intéresse.
2. En pleine réception, il passe au code d'un écran puis revient, sans changer d'adresse.

## Besoins

| ID | Besoin | Priorité |
|---|---|---|
| [[besoins/B-017]] | Ouvrir tout un projet d'une seule commande et naviguer entre ses usages | must |

## Critères de succès de la version

`wikik` seul lance tout ; les anciennes sous-commandes n'existent plus ; l'accueil, la barre d'activité et les accueils de section s'affichent sans erreur (clair, sombre, téléphone).

## Hypothèses et risques

- Un serveur qui écrit peut être exposé au réseau : même règle qu'avant (`[serveur] reseau`, réseau de confiance), et le site public reste l'export statique.

## Décision de lancement

Accordée par Tibo le 19/10/2026 (« on fait qu'un seul serveur … une seule commande wikik »).
