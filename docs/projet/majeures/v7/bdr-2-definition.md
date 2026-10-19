---
id: BDR-v7-2
type: bdr-definition
titre: "Définition du besoin v7 (retours de la réception de la v6)"
statut: proposé
version: v7
sources: [BDR-v7-1]
---

# BDR v7, partie 2 : définition du besoin

## Objectifs mesurables

- Les 39 remarques du périmètre sont traitées, chacune avec son commit, dans le cahier de réception.
- Tibo lance le traitement d'une réception depuis wikik, et l'agent commence par un plan.

## Périmètre (inclus / exclu)

**Inclus** : noms et navigation, graphe, récapitulatifs Produit et Projet, liens des documents, clarifications et parcours, complétion de la carte, paramètres de l'IA et terminal intégré.

**Exclu** : la réécriture du front en Rust / WebAssembly (REC-009.2), qui fait l'objet de la v8 (voir le plan v7, section « Décision »).

## Cas d'usage prioritaires

1. Tibo ouvre un plan et voit ses phases en colonnes ; il clique sur un identifiant cité et arrive dessus.
2. Il lance une clarification d'un besoin et tombe directement sur des pistes qu'il garde, corrige ou rejette.
3. Il termine une réception, clique sur « Traiter les remarques » : la consigne part dans le terminal où tourne l'agent, qui répond par un plan.

## Besoins

| ID | Besoin | Priorité |
|---|---|---|
| [[besoins/B-017]] | Ouvrir tout un projet d'une seule commande et naviguer entre ses usages (repris) | must |
| [[besoins/B-018]] | Régler l'IA de wikik et piloter l'agent de développement depuis le projet | must |

## Critères de succès de la version

Les remarques du périmètre sont au statut « traitée » ; `make check` et `wikik check` passent ; contrôle headless clair, sombre et téléphone sans erreur.

## Hypothèses et risques

- Un terminal dans le navigateur est un accès shell : il n'est ouvert qu'à la machine locale, ou avec la clé affichée au lancement (ADR-0013).

## Décision de lancement

Accordée par Tibo le 19/10/2026 (« planifie leur traitement … tu lanceras la mise à jour dans une branche temporaire »).
