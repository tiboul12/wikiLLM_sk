---
id: ADR-0012
type: adr
titre: "Un seul serveur par projet, une seule commande : wikik"
statut: accepté
date: 2026-10-19
remplace: [ADR-0009]
exigences: [EF-130, EF-131]
---

# ADR-0012 : Un seul serveur par projet, une seule commande : `wikik`

## Contexte

En v4 et v5, chaque usage avait son serveur : `wikik serve` (wiki, 8484), `wikik atelier` (8485), `wikik reception` (8487). À l'usage, Tibo a trouvé le lancement « usine à gaz », et une page d'accueil commune avec une barre de sections (à la VS Code) tomberait sur des pages mortes dès qu'un des trois serveurs n'est pas lancé. Tibo ne veut plus qu'une commande, sans alias historiques qui poseraient problème plus tard.

## Décision

- **`wikik` sans sous-commande lance le serveur du projet** : wiki (Produit, Projet), atelier et cahier de réception, sur un seul port (8484 par défaut, ou le suivant libre ; `[serveur] reseau` comme avant).
- **`serve`, `atelier` et `reception` sont supprimées**, sans alias. Les sous-commandes outils restent (`query`, `check`, `mcp`, `init`, `export`, `relu`, `nouveau`, `stats`, `index`) : ce ne sont pas des serveurs.
- Le serveur écrit **seulement dans `docs/atelier/`, `docs/projet/` et `docs/reception/`**, par des chemins calculés côté serveur, avec les gardes de l'atelier (corps JSON, origine identique). Lancer le serveur n'écrit rien dans le dépôt : la carte de réception est enregistrée à la première remarque ou correction.
- **Le site exporté** (`wikik export`) reste en lecture seule et ne montre que Produit et Projet.
- Les adresses `[atelier] url` et `url_wiki` disparaissent : tous les liens sont internes ; l'adresse renvoyée par `atelier_rappel` vient du port réellement pris.

## Conséquences

- Remplace ADR-0009 (atelier séparé du wiki) : la séparation lecture / écriture ne passe plus par deux serveurs mais par les routes ; le site public reste l'export statique.
- Une seule adresse par projet ; l'accueil et la barre d'activité mènent partout sans lien mort.

## Alternatives écartées

- **Garder trois serveurs et relier leurs pages** : liens morts, trois ports par projet.
- **Garder `serve` / `atelier` / `reception` comme alias** : refusé par Tibo (commandes historiques à maintenir).
