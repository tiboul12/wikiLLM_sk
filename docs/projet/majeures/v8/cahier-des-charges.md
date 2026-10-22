---
id: CDC-v8
type: cahier-des-charges
titre: "Cahier des charges v8 (retours de la réception de la v7)"
statut: proposé
version: v8
sources: [BDR-v8-2]
---

# Cahier des charges v8

## Exigences fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/EF-150]] | Outils regroupés, accordéons fermés à l'arrivée, replier et déplier | must | B-019 |
| [[exigences/EF-151]] | Onglets de pages, détachables | must | B-019 |
| [[exigences/EF-152]] | Recherche avec champ et recherche assistée par IA | should | B-019 |
| [[exigences/EF-153]] | Graphe chargé une fois, rafraîchi, cherchable, avec les liens entre fichiers et entre pages | must | B-001 |
| [[exigences/EF-154]] | Page fichier rapide | should | B-001 |
| [[exigences/EF-155]] | Connexion au LLM : fournisseur, modèle, effort | must | B-021 |
| [[exigences/EF-156]] | Prompts système en skills | must | B-021 |
| [[exigences/EF-157]] | Gestion : plans, actions en attente, favoris ; compteurs remontés | must | B-019 |
| [[exigences/EF-158]] | Relecture : valider, corriger, éditer ; seulement ce qui compte | must | B-020 |
| [[exigences/EF-159]] | Annuler et rétablir (Ctrl+Z, Ctrl+Y) | should | B-020 |
| [[exigences/EF-160]] | Cascade besoin, cahier des charges, exigence, test, ticket | must | B-020 |
| [[exigences/EF-161]] | Ateliers en sessions, clarifications à part | should | B-022 |
| [[exigences/EF-162]] | Clarification : hors-sujet détecté, pages et fichiers liés, même écran que la relecture | should | B-022 |
| [[exigences/EF-163]] | Bilan limité aux besoins du parcours | must | B-022 |
| [[exigences/EF-164]] | Carte de réception complète pour les étapes de l'atelier | should | B-023 |
| [[exigences/EF-165]] | Validation des remarques et état git des corrections | must | B-023 |
| [[exigences/EF-166]] | Historique git | must | B-023 |
| [[exigences/EF-167]] | Dessin : carte mentale en Mermaid, storyboard en SVG | could | B-022 |
| [[exigences/EF-168]] | Terminal ouvert dans le dossier du projet | must | B-018 |

## Exigences non fonctionnelles

ENF-004 (local) s'applique ; l'écriture de fichiers depuis le navigateur suit ADR-0013 et ADR-0016.

## Exigences modifiées ou abandonnées par rapport à la version précédente

- EF-144 : les clarifications quittent le volet Projet pour le Design thinking (REC-072, REC-074).
- EF-147 : la section Paramètres est refondue (EF-155, EF-156) ; les consignes deviennent des skills.
- Backlog « Édition dans le navigateur pour la zone Projet (Q5 : non en v1) » : faite (EF-158).
