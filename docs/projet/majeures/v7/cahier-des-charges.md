---
id: CDC-v7
type: cahier-des-charges
titre: "Cahier des charges v7 (retours de la réception de la v6)"
statut: proposé
version: v7
sources: [BDR-v7-2]
---

# Cahier des charges v7

## Exigences fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/EF-140]] | Noms et navigation qui suivent la façon de travailler | must | B-017 |
| [[exigences/EF-141]] | Graphe des liens par catégorie, lisible et stable | must | B-001 |
| [[exigences/EF-142]] | Récapitulatifs qui mènent aux éléments | must | B-003, B-006 |
| [[exigences/EF-143]] | Gestion de projet et liens des documents | must | B-006, B-014 |
| [[exigences/EF-144]] | Clarifications regroupées par sujet, sans saisie préalable | must | B-013 |
| [[exigences/EF-145]] | Plusieurs parcours par projet | should | B-012 |
| [[exigences/EF-146]] | Complétion de la carte expliquée et annulable | should | B-016 |
| [[exigences/EF-147]] | Paramètres : IA et consignes | must | B-018 |
| [[exigences/EF-148]] | Terminal intégré et envoi du traitement | must | B-018 |

## Exigences non fonctionnelles

ENF-004 (local) s'applique ; le terminal et l'envoi au terminal suivent ADR-0013.

## Exigences modifiées ou abandonnées par rapport à la version précédente

- SPEC-21 R4 : la clarification ne demande plus de question ni de conclusion ; les décisions sur les pistes font la réponse.
- SPEC-23 R7 : l'adresse de l'application n'est plus saisie sur le cahier ; elle est déduite, ou réglée dans les paramètres.
