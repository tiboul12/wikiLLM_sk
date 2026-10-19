---
id: ADR-0009
type: adr
titre: "Atelier : serveur séparé du wiki"
statut: remplacé
date: 2026-10-17
remplace: []
remplace_par: [ADR-0012]
exigences: []
---

# ADR-0009 : Atelier : serveur séparé du wiki

## Contexte

L'atelier de définition du besoin (v4) écrit des fichiers et appelle un LLM. Le wiki (`wikik serve`) est en lecture seule par conception (ENF-004, SPEC-09) : aucune route d'écriture, aucun appel réseau sortant. Tibo veut un atelier vivant, qu'on rouvre en plein développement (décision B2 du [[majeures/v4/cadrage]]).

## Décision

- L'atelier est une **commande séparée du même binaire** : `wikik atelier`, avec son propre port (8485 par défaut), son interface et une API d'écriture.
- Le **wiki reste en lecture seule** ; les deux se relient par des liens (« ouvrir dans l'atelier », « voir dans le wiki »).
- L'atelier écoute sur la **boucle locale** par défaut ; `--autoriser-reseau` est exigé pour une autre adresse, comme pour `serve`.
- L'atelier **n'écrit que dans `docs/atelier/` et `docs/projet/`**, par des chemins calculés côté serveur, jamais fournis par le client.

## Conséquences

- Les garanties du wiki (SPEC-09) ne changent pas.
- Un seul binaire à installer ; l'atelier partage l'index, le moteur de requêtes et le front du wiki.

## Alternatives écartées

- **Mode écriture dans `wikik serve`** : mêlerait lecture publique et écriture locale dans un même serveur.
- **Application de bureau à part (Tauri)** : un second outil à installer ; possible plus tard au-dessus de la même API.
