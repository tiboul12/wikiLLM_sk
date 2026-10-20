---
id: BDR-v7-1
type: bdr-analyse
titre: "Analyse du besoin v7 (retours de la réception de la v6)"
statut: proposé
version: v7
sources: [BDR-v6-2, REC-009.2, REC-015, REC-019, REC-022, REC-024.2, REC-041]
---

# BDR v7, partie 1 : analyse du besoin

> Aucune solution technique dans ce document. Matière : les 40 remarques du cahier de réception que Tibo a saisies sur wikik lui-même le 19/10/2026 (`docs/reception/remarques/`).

## Contexte

Première réception complète de wikik par son utilisateur, écran par écran, avec captures. Toutes les remarques sont « gênantes » ; aucune n'est bloquante.

## Personnes concernées

Tibo, qui lit son projet, définit le besoin, réceptionne et fait développer par un agent ; plus tard, l'équipe.

## Situation actuelle et irritants (avec preuves)

| Irritant | Preuve |
|---|---|
| Les noms ne disent pas ce que sont les écrans. | REC-014 (« Tableau » ouvre l'atelier, qui doit s'appeler Design thinking), REC-027, REC-028 (carte, page, cahier de réception confondus), REC-036 (« je ne sais absolument pas de quoi traite cette section »). |
| La navigation ne suit pas la façon de travailler. | REC-015 (ordre Projet, Design thinking, Produit, Réception), REC-016, REC-033, REC-008, REC-010 (replier tout), REC-011 (volet trop étroit), REC-026 (structure des pages invisible), REC-037 (l'élément choisi reste caché). |
| Le graphe ne montre pas les liens qui comptent. | la remarque sur le graphe « tout en vert », sans code ni pages ni remarques (supprimée depuis par Tibo), REC-002, REC-004 (zones denses illisibles), REC-003, REC-005 (rien au clic ni au survol), REC-006 (orientation différente à chaque ouverture). |
| On voit des chiffres sans pouvoir agir dessus. | REC-007 (plus de récapitulatif par fichier), REC-012, REC-013 (« j'en ai 49 à relire, mais lesquels »), REC-023 (ce qui ne va pas dans un fichier). |
| Les documents de gestion sont noyés et les références mortes. | REC-017.2 (backlog et plans perdus dans « Autres »), REC-018.2 (identifiants cités sans lien, coûteux pour le LLM), REC-020, REC-021 (ancres et liens cassés). |
| La clarification est incompréhensible et se bloque. | REC-024.2, REC-025 (deux écrans, une question à écrire, une conclusion inutile), REC-032, REC-034, REC-035 (impossible de supprimer ou de reprendre), REC-040.2 (pas de regroupement ni d'état par sujet). |
| Un seul parcours par projet. | REC-039. |
| Le LLM de la réception agit sans qu'on sache quoi, sans arrêt possible. | REC-030, REC-031. |
| Rien ne relie wikik à l'agent qui développe. | REC-019 (choisir l'IA, reprendre ses consignes), REC-022 (terminal et IA dans un volet), REC-041 (« Traiter les remarques » doit partir dans le terminal, avec une consigne qui impose un plan). |
| Le front n'est pas en Rust. | REC-009.2 (« je veux du full Rust / WebAssembly »). |

## Contraintes

- Une seule commande `wikik` (ADR-0012) ; pas d'« usine à gaz ».
- Le serveur peut écouter le réseau local (`[serveur] reseau = true` sur ce dépôt) : tout ce qui exécute des commandes doit rester réservé à Tibo.

## Hors sujet

- Le kit UI commun à tous les projets de Tibo (« je ferai plus tard une passe »).
