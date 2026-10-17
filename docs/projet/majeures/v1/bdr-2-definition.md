---
id: BDR-v1-2
type: bdr-definition
titre: Définition du besoin v1
statut: proposé
version: v1
sources: [BDR-v1-1]
---

# BDR v1, partie 2 : définition du besoin

## Objectifs mesurables

1. Sur une tâche réelle d'ATS, l'agent obtient le contexte nécessaire **en ouvrant moins de fichiers source et en consommant moins de tokens** qu'une session sans l'outil (mesure avant / après sur 3 à 5 tâches).
2. **Toute fonction dont le code change sans que sa doc suive est signalée**, sans faux négatif sur les cas de test.
3. **Chaque exigence `must` de cette version est reliée à du code ou à un test** qui la cite.

## Périmètre

| Inclus | Exclu (voir `wiki/taches/backlog.md`) |
|---|---|
| Projets Rust ; wikiK et ATS comme terrains | TypeScript, Python |
| Doc dans le code, index, détection de doc périmée | CI de doc, publication en équipe |
| Requêtes pour l'agent en ligne de commande, skill | Serveur MCP, recherche sémantique |
| Wiki local en lecture | Édition dans le navigateur, site statique, WASM |
| Zone Projet : BDR, besoins, exigences, ADR, specs | Atelier de conception |

## Cas d'usage prioritaires

1. **Avant de modifier une fonction**, l'agent demande ce qu'elle fait, qui l'appelle, et quelles décisions et exigences la concernent.
2. **En début de session**, l'agent obtient une carte du projet en quelques centaines de lignes.
3. **Après une modification**, Tibo ou l'agent voit quelles docs sont devenues fausses.
4. **Sur un projet déjà écrit**, l'agent propose les blocs manquants module par module, et Tibo relit.
5. **Pour comprendre un projet**, Tibo lit dans le navigateur le code documenté et les documents du projet, reliés entre eux.

## Besoins

| ID | Besoin |
|---|---|
| [[besoins/B-001]] | Comprendre la structure d'un projet sans relire ses fichiers source. |
| [[besoins/B-002]] | Savoir ce qu'une modification va toucher avant de la faire. |
| [[besoins/B-003]] | Savoir quand une doc ne correspond plus au code. |
| [[besoins/B-004]] | Relier décisions et exigences au code qui les applique, dans les deux sens. |
| [[besoins/B-005]] | Écrire la doc une seule fois, près du code, avec une convention commune à l'humain et à l'agent. |
| [[besoins/B-006]] | Lire le projet documenté dans un navigateur. |
| [[besoins/B-007]] | Adopter l'outil sur un projet existant progressivement. |
| [[besoins/B-008]] | Pouvoir passer en équipe plus tard sans tout refaire. |
| [[besoins/B-009]] | Encadrer chaque mise à jour majeure par une BDR et un cahier des charges. |

## Critères de succès de la version

Ceux de la « définition de fini » du plan (`docs/projet/plan-v1.md`, section 1).

## Hypothèses et risques

- **Hypothèse** : une doc courte et ciblée par symbole suffit à l'agent dans la majorité des tâches. À vérifier par la mesure de la Phase 7.
- **Hypothèse** : l'analyse syntaxique (sans types) donne un graphe d'appels assez fiable pour `impact`. SPIKE-01 : 21 % des appels se résolvent directement ; les autres sont marqués « probables ».
- **Risque** : la doc dérive vers la paraphrase du code. Parade : convention SPEC-01 avec contre-exemples.

## Décision de lancement

**Go**, le 17/10/2026, par délégation de Tibo (exécution en continu demandée) : à confirmer à sa relecture.
