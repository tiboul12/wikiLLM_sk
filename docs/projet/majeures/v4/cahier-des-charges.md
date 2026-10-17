---
id: CDC-v4
type: cahier-des-charges
titre: "Cahier des charges v4 (atelier de définition du besoin)"
statut: proposé
version: v4
sources: [BDR-v4-2]
---

# Cahier des charges v4

## Exigences fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/EF-100]] | Lancer le serveur de l'atelier | must | B-012 |
| [[exigences/EF-101]] | Conduire un parcours d'étapes | must | B-012 |
| [[exigences/EF-102]] | Décrire chaque technique par une fiche | must | B-012 |
| [[exigences/EF-103]] | Proposer en lot, choisir à l'écran | must | B-012 |
| [[exigences/EF-104]] | Appeler le LLM par le CLI local, branché sur wikik | must | B-012 |
| [[exigences/EF-105]] | Écrire les artefacts en fichiers traçables | must | B-014 |
| [[exigences/EF-106]] | Lire l'atelier dans l'index, le wiki et le MCP | must | B-013, B-014 |
| [[exigences/EF-107]] | Produire la doc du projet | must | B-012, B-014 |
| [[exigences/EF-108]] | Vérifier la traçabilité | should | B-014 |
| [[exigences/EF-109]] | Relier le wiki et l'atelier | should | B-013 |
| [[exigences/EF-110]] | Rappeler l'atelier depuis le développement | must | B-013 |
| [[exigences/EF-111]] | Afficher la consommation de l'abonnement | should | B-012 |
| [[exigences/EF-112]] | Écrans dédiés au mode Express | should | B-012 |

## Exigences non fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/ENF-007]] | Attente d'une proposition | must | B-012 |
| [[exigences/ENF-008]] | Isolement du LLM | must | B-012 |

ENF-002 (déterminisme) et ENF-004 (sécurité) restent valables ; l'atelier n'est pas exposé sur le réseau par défaut.

## Exigences modifiées ou abandonnées

Aucune.
