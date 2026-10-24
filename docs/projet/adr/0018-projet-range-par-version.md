---
id: ADR-0018
type: adr
titre: "La zone Projet se range par version, déduite des documents existants"
statut: proposé
date: 2026-10-24
exigences: [EF-174, EF-175]
---

# ADR-0018 : la zone Projet se range par version, déduite des documents existants

## Contexte

Le volet Projet range les documents par catégorie (BDR, cahier des charges, besoins, exigences, tests…). Avec neuf versions, chaque catégorie mélange toutes les versions : « ce n'est lisible ni pour moi ni pour le modèle » (REC-101.2). Tibo veut voir d'un seul tenant la structure d'une version : la demande d'évolution, l'analyse, la définition, le cahier des charges, ses besoins, leurs exigences et leurs tests.

## Décision

- **Une version par document, déduite** : le champ `version` du frontmatter (déjà présent sur les BDR, cahiers des charges, besoins, exigences, tests, specs, plans) ; une mineure (`v1.1`) se range sous sa majeure (`v1`). Une ADR, qui n'a pas de version, prend celle de sa première exigence (`exigences:`). Aucun champ nouveau n'est demandé.
- **Arbre d'une version**, dans l'ordre de la démarche : demande d'évolution (les `sources` de la BDR analyse : remarques de réception, parcours d'atelier, version précédente), analyse (BDR 1), définition (BDR 2), cahier des charges, puis les besoins de la version avec leurs exigences et leurs tests ; les exigences de la version qui couvrent un besoin d'une version antérieure sont rangées sous ce besoin, marqué de sa version ; puis les exigences non fonctionnelles, les décisions, les specs, le plan et les autres documents (mesures, spikes, cadrage).
- **Transverse** : ce qui n'a pas de version (backlog, ADR sans exigence) reste à part, avec la gestion en tête du volet.
- Le même arbre sert au volet et à l'accueil Projet, où il est accompagné de l'état de suivi de chaque version (à relire, validés, tests validés) : sur téléphone, l'accueil suffit pour atteindre chaque document.
- Les catégories à plat disparaissent du volet.

## Conséquences

- Un document sans version (nouveau document mal rempli) tombe dans « Transverse » : il se voit, et `version:` se corrige dans l'éditeur.
- Le résumé de document servi au front porte ses `sources`.

## Alternatives écartées

- **Un dossier par version pour tous les documents** (`majeures/v9/exigences/…`) : déplacerait 200 fichiers, casserait les liens et l'historique ; les identifiants restent uniques sur tout le projet.
- **Un nouveau type « demande d'évolution »** : la demande existe déjà sous forme de remarques de réception ou de parcours d'atelier, cités par la BDR analyse ; un document de plus doublerait l'information.
