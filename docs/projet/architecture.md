---
id: ARCHITECTURE
type: architecture
titre: "Architecture de wikik v1"
statut: à jour
version: v1
decisions: [ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006]
---

# Architecture de wikik v1

> Le détail de chaque module est dans son en-tête `//!` : `wikik query carte` ou la zone Produit du wiki. Cette page donne la vue d'ensemble et les flux.

## Composants

```mermaid
flowchart LR
  subgraph Depot["Dépôt du projet (versionné)"]
    SRC["Code + blocs /// //!"]
    PROJ["docs/projet/*.md (frontmatter)"]
    CFG["wikik.toml"]
  end
  subgraph Core["wikik-core (bibliothèque, sans réseau)"]
    LANG["lang/rust.rs : tree-sitter"]
    PRJ["projet.rs : frontmatter, liens"]
    IDX[("index.rs : .wikik/index.sqlite")]
    RES["resolution.rs : arêtes"]
    CHK["check.rs + git.rs"]
    QRY["query.rs"]
    WIKI["wiki.rs : données des pages"]
  end
  subgraph Bin["wikik (binaire)"]
    CLI["main.rs : CLI"]
    SRV["serve.rs : axum"]
    FRONT["front/ Svelte (embarqué)"]
  end
  SRC --> LANG --> IDX
  PROJ --> PRJ --> IDX
  CFG --> IDX
  IDX --> RES --> IDX
  IDX --> CHK
  IDX --> QRY --> CLI
  CHK --> CLI
  IDX --> WIKI --> SRV --> FRONT
```

## Flux d'une requête

1. `Index::ouvrir` : ouvre ou recrée l'index si le schéma ou les règles d'extraction ont changé ([[adr/0003-index-sqlite-non-versionne]]).
2. `Index::mettre_a_jour` : liste les fichiers (`.gitignore` et `wikik.toml` respectés), ne relit que ceux dont la taille, la date puis le hash ont changé, recalcule toutes les arêtes si quelque chose a changé ([[SPEC-03]]).
3. La requête lit SQLite ; `check` et le statut de doc interrogent git pour la référence de chaque bloc ([[adr/0006-perime-reference-git]]).

## Règles transverses

- **Déterminisme** : dump JSON trié, sans horodatage ; testé par instantané ([[ENF-002]]).
- **Une seule écriture dans le code** : `wikik relu`, qui ne touche qu'une ligne `@relu`.
- **Aucune écriture par le réseau** : le serveur n'a que des routes `GET` ([[SPEC-09]]).
- **Le front n'a pas de logique métier** : il affiche les structures de `wiki.rs`, dont les types TypeScript sont générés ([[adr/0004-front-typescript-wasm-reserve]]).
