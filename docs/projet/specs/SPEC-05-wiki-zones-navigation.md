---
id: SPEC-05
type: spec
titre: "Wiki : zones Produit et Projet, navigation"
statut: proposé
version: v1
exigences: [EF-041, EF-042, EF-043, EF-044, EF-045]
decisions: [ADR-0004]
---

# SPEC-05 : Wiki, zones et navigation

## Problème

Lire le projet documenté dans un navigateur : le code et sa doc (zone Produit), les documents du projet (zone Projet), et les liens entre les deux, sans jamais éditer depuis le wiki.

## Exigences

1. **R1** Deux zones dans la barre latérale : **Produit** (arbre des fichiers de code) et **Projet** (documents groupés par type : plan, BDR, besoins, exigences, ADR, specs, autres).
2. **R2** Page **fichier** : en-tête `//!` rendu, puis une **cellule par symbole**, comme une cellule de notebook : doc rendue en Markdown, signature, statut de doc (à jour / périmé / sans doc), balises en pastilles cliquables (`@adr` → ADR, `@req` → exigence), appelants et appelés cliquables, code **replié** et coloré.
3. **R3** Page **document** : frontmatter en tête (statut, priorité, besoins, etc.), corps Markdown avec Mermaid, liens `[[...]]` résolus par l'index (cassés en rouge), puis « cité par » : symboles qui le citent via `@adr`/`@req`.
4. **R4** **Graphe** : graphe local d'un symbole (appelants, appelés, ADR, exigences, profondeur 1 à 2) et graphe des documents (liens). Les arêtes probables sont en pointillé.
5. **R5** **Recherche** plein texte (FTS5) dans une barre en haut.
6. **R6** **Hooks** ⚓ : chaque cellule de symbole, chaque document et chaque titre de section a un bouton qui copie une référence `@wiki:` résolue par `wikik query ref`.
7. **R7** URL stables : `/produit/<chemin>#<nom qualifié>`, `/projet/<id>`, `/recherche?q=…`, `/graphe/<id>`.
8. **R8** Thème clair et sombre suivant le système ; mise en page lisible sur un écran de téléphone.

## Proposition

- **Framework** : Svelte 5 + TypeScript, compilé par Vite (ADR-0004 laissait Svelte par défaut, Leptos à rediscuter). Choix pris par délégation le 17/10 ; si Tibo préfère Leptos, seul `front/` change, l'API reste.
- **Bibliothèques** (dans le binaire, aucun CDN) : `marked` (Markdown), `DOMPurify` (assainissement), `highlight.js` (coloration), `mermaid` (chargé à la demande), `cytoscape` (graphes).
- **Types** : les réponses de l'API sont des structures Rust exportées en TypeScript par `ts-rs` dans `front/src/types/`.

## Alternatives écartées

- **Reprendre `legacy/index.html`** : CDN, HTML non assaini, logique d'édition.
- **Rendu côté serveur** : il faudrait un moteur de templates et on perdrait le graphe interactif.

## Critères d'acceptation

1. La page d'un fichier d'ATS montre chaque symbole avec sa doc et son code repliable (EF-041).
2. La page d'une exigence de wikiK liste les symboles qui la citent, cliquables (EF-042).
3. Une page contenant `<script>alert(1)</script>` ne l'exécute pas (ENF-004).
