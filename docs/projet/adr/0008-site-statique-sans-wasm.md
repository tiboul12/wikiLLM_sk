---
id: ADR-0008
type: adr
titre: "Site statique : données JSON pré-calculées, sans WebAssembly"
statut: accepté
date: 2026-10-17
remplace: []
exigences: [EF-080]
---

# ADR-0008 : Site statique sans WebAssembly

## Contexte

L'ADR-0004 réservait WebAssembly au moteur de requêtes du site publié. En construisant l'export (v3), il faut choisir comment le site statique répond aux pages, à la recherche et au graphe.

## Décision

- L'export **pré-calcule** toutes les pages en JSON (`donnees/`) : accueil, une page par fichier, une par document, le rapport de `check`, le graphe complet et un index de recherche.
- La **recherche** et le **graphe local** sont calculés dans le navigateur sur ces données (quelques centaines de Ko pour un projet comme wikiK ou ATS).
- Le front détecte le mode statique (balise `<meta name="wikik-statique">`) et utilise un routage par ancre (`#/produit/…`), qui fonctionne sous n'importe quel chemin d'hébergement sans configuration du serveur.
- WebAssembly n'est **pas** utilisé. SQLite compilé en WASM alourdirait le site (plus d'1 Mo) et la chaîne de compilation, pour un gain nul à cette taille.

## Conséquences

- Le site exporté est un dossier de fichiers, publiable tel quel (GitHub Pages, serveur de fichiers).
- La recherche statique est plus simple que FTS5 (correspondance de mots, sans classement bm25) ; acceptable pour la lecture.
- À réévaluer si un projet dépasse quelques dizaines de milliers de symboles : c'est là que WASM redeviendrait utile.

## Alternatives écartées

- **Moteur de requêtes en WASM** (ADR-0004) : voir plus haut ; reste possible plus tard sans changer le format des données.
- **Rendu HTML côté serveur au moment de l'export** : perdrait le graphe et la recherche interactifs, et dupliquerait le rendu du front.
