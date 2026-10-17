---
id: ADR-0002
type: adr
titre: "Cœur en Rust, livré en un seul binaire"
statut: accepté
date: 2026-10-17
remplace: [ADR-0001]
exigences: [ENF-003, ENF-005, EF-001]
---

# ADR-0002 : Cœur en Rust, livré en un seul binaire

## Contexte

La refonte doit analyser le code avec tree-sitter (décision D3), indexer vite et incrémentalement, et se distribuer facilement le jour où une équipe l'utilise. Les projets cibles sont surtout en Rust (ATS) et TypeScript (shelley) ; l'équipe apprécie Rust. La v0 était en Python standard sans dépendance (ADR-0001).

## Décision

- Tout le cœur est en **Rust** : extraction, index, check, requêtes, serveur de prévisualisation. Un workspace de deux crates : `wikik-core` (bibliothèque) et `wikik` (binaire).
- **Politique de dépendances** : les crates sont assumées tant qu'elles sont compilées dans le binaire ; l'utilisateur n'installe rien d'autre que l'exécutable (SQLite inclus, front embarqué). Dépendances de départ : `tree-sitter`, `tree-sitter-rust`, `rusqlite` (`bundled`), `clap`, `serde`, `serde_json`, `serde_yaml_ng`, `toml`, `ignore`, `sha2` ; `axum`, `tokio`, `rust-embed` pour `serve`.
- La v0 Python n'est pas portée ; elle reste au tag `v0-python`.

## Conséquences

- Analyse rapide (SPIKE-01 : 16 µs par ligne) et binaire autonome (ENF-005).
- Compilation plus longue qu'un script : à compenser par des crates petites et des tests ciblés.
- Le même cœur pourra être compilé en WebAssembly pour le futur site statique (ADR-0004).

## Alternatives écartées

- **Python + tree-sitter** : environnement à installer, plus lent, et les projets cibles ne sont pas en Python.
- **Python `ast` sans dépendance** : Python seulement, ne couvre aucun projet cible.
