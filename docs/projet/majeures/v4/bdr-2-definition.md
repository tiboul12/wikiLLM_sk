---
id: BDR-v4-2
type: bdr-definition
titre: "Définition du besoin v4 (atelier de définition du besoin)"
statut: proposé
version: v4
sources: [BDR-v4-1]
---

# BDR v4, partie 2 : définition du besoin

## Objectifs mesurables

1. Une séance Express conduit un projet du pitch au cahier des charges et à la roadmap en **une soirée**, avec **moins d'une trentaine de décisions** à l'écran (garder, corriger, rejeter), au lieu de centaines de messages.
2. **Chaque exigence produite remonte** à un besoin, à un storyboard ou une analyse, et à l'idée d'origine ; `wikik check` le vérifie.
3. En développement, l'agent de code **rouvre l'atelier sur un identifiant** (besoin, exigence) en une commande, et la clarification reste attachée au besoin.

## Périmètre

| Inclus | Exclu |
|---|---|
| Atelier du pitch à la roadmap (B7), modes Express et Complet (B6) | Specs, ADR, code (Claude Code) |
| Fiches déclaratives des 11 techniques | Écrans dédiés du mode Complet (au-delà de l'écran générique) |
| Fichiers du dépôt, lus par l'index, le wiki et le MCP (B3) | Base de données propre à l'atelier |
| LLM par le CLI local branché sur le MCP de wikik (B4) | API avec clé (possible plus tard) |

## Besoins

| ID | Besoin |
|---|---|
| [[besoins/B-012]] | Définir le besoin d'un projet avec un atelier guidé, sans les lourdeurs du chat. |
| [[besoins/B-013]] | Revenir sur un besoin pendant le développement pour l'éclaircir. |
| [[besoins/B-014]] | Suivre chaque exigence jusqu'à l'idée qui l'a fait naître. |

## Critères de succès

Ceux de la définition de fini de [[majeures/v4/plan-v4]].

## Hypothèses et risques

- **Hypothèse** : 6 à 10 s par proposition (SPIKE-02) restent acceptables si les variantes arrivent en lot.
- **Risque** : réécriture infidèle du framework ; parade : exemples « MyRandonnée » rejoués.

## Décision de lancement

**Go**, le 18/10/2026, par délégation de Tibo (« on attaque, lance tout ») : à confirmer à sa relecture.
