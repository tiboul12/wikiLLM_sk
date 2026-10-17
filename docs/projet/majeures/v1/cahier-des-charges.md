---
id: CDC-v1
type: cahier-des-charges
titre: Cahier des charges v1
statut: proposé
version: v1
sources: [BDR-v1-2]
---

# Cahier des charges v1

> Chaque exigence a son fichier dans `docs/projet/exigences/` (énoncé et critère d'acceptation). Cette page n'en est que la liste.
> 34 exigences fonctionnelles (21 must, 12 should, 1 could) et 6 non fonctionnelles.

## Exigences fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/EF-001]] | Construire l'index d'un projet Rust | must | B-001 |
| [[exigences/EF-002]] | Mettre à jour l'index de façon incrémentale | must | B-001 |
| [[exigences/EF-003]] | Produire un dump JSON déterministe de l'index | must | B-008 |
| [[exigences/EF-004]] | Lire une configuration de projet optionnelle | must | B-008 |
| [[exigences/EF-005]] | Indexer la zone Projet | must | B-004, B-009 |
| [[exigences/EF-006]] | Extraire les balises des blocs de doc | must | B-004 |
| [[exigences/EF-007]] | Repérer les composants Leptos utilisés dans `view!` | should | B-001, B-002 |
| [[exigences/EF-010]] | Détecter la doc périmée | must | B-003 |
| [[exigences/EF-011]] | Signaler les blocs manquants | must | B-005, B-007 |
| [[exigences/EF-012]] | Signaler les références cassées | must | B-004 |
| [[exigences/EF-013]] | Signaler les exigences non couvertes | should | B-009 |
| [[exigences/EF-014]] | Produire une sortie texte et JSON avec code de retour | must | B-008 |
| [[exigences/EF-015]] | Marquer un bloc comme relu | should | B-003 |
| [[exigences/EF-020]] | Requête carte | must | B-001 |
| [[exigences/EF-021]] | Requête fichier | must | B-001 |
| [[exigences/EF-022]] | Requête symbole | must | B-001, B-004 |
| [[exigences/EF-023]] | Requête impact | must | B-002 |
| [[exigences/EF-024]] | Requête périmés | must | B-003 |
| [[exigences/EF-025]] | Requête cherche | should | B-001 |
| [[exigences/EF-026]] | Requête adr | should | B-004 |
| [[exigences/EF-027]] | Requête exigence | should | B-004, B-009 |
| [[exigences/EF-028]] | Résoudre les références @wiki: | could | B-001 |
| [[exigences/EF-029]] | Mettre l'index à jour avant de répondre | must | B-001 |
| [[exigences/EF-030]] | Fournir un skill pour l'agent | must | B-001, B-005 |
| [[exigences/EF-031]] | Procédure d'amorçage d'un projet existant | should | B-007 |
| [[exigences/EF-032]] | Procédures ADR et mise à jour majeure | should | B-009 |
| [[exigences/EF-033]] | Initialiser un projet | should | B-007, B-009 |
| [[exigences/EF-040]] | Servir le wiki en local | must | B-006 |
| [[exigences/EF-041]] | Afficher la zone Produit | must | B-006 |
| [[exigences/EF-042]] | Afficher la zone Projet | must | B-006, B-004 |
| [[exigences/EF-043]] | Afficher un graphe | should | B-006, B-002 |
| [[exigences/EF-044]] | Rechercher dans le wiki | should | B-006 |
| [[exigences/EF-045]] | Copier une référence d'élément | should | B-001, B-006 |
| [[exigences/EF-050]] | Migrer le wiki v0 de wikiK | must | B-005 |

## Exigences non fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/ENF-001]] | Budget de tokens des requêtes | must | B-001 |
| [[exigences/ENF-002]] | Déterminisme | must | B-008 |
| [[exigences/ENF-003]] | Vitesse | must | B-001 |
| [[exigences/ENF-004]] | Sécurité locale | must | B-008 |
| [[exigences/ENF-005]] | Distribution en un seul binaire | must | B-008 |
| [[exigences/ENF-006]] | Robustesse aux erreurs de parse | must | B-007 |

## Exigences modifiées ou abandonnées par rapport à la version précédente

Aucune : première version suivie par ce cycle.
