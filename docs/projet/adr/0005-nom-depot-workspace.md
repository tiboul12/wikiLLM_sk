---
id: ADR-0005
type: adr
titre: "Nom de travail wikik, refonte dans ce dépôt, workspace en deux crates"
statut: accepté
date: 2026-10-17
remplace: []
exigences: [ENF-005]
---

# ADR-0005 : Nom, dépôt et organisation

## Contexte

Question Q6 de la roadmap (nom du skill) et choix entre nouveau dépôt et refonte sur place.

## Décision

- **Nom** : `wikik` pour le binaire, `wikik.toml` pour la configuration, `.wikik/` pour l'index. Nom de travail confirmé par délégation le 17/10 ; un renommage avant la v1.0 reste possible et ne coûte qu'un remplacement de chaînes.
- **Dépôt** : la refonte se fait dans ce dépôt (`wikiK`, remote `tiboul12/wiki_LLM`), pour garder l'historique et le wiki v0 comme matière de migration.
- **Organisation** : workspace Cargo `crates/wikik-core` (bibliothèque sans entrée/sortie réseau) et `crates/wikik` (CLI et serveur) ; `front/`, `skill/`, `templates/`, `tests/fixtures/`, `docs/projet/`.

## Conséquences

- Le cœur reste réutilisable (MCP, WASM) sans dépendre de la CLI.
- `legacy/` coexiste jusqu'à la Phase 9.

## Alternatives écartées

- **Nouveau dépôt** : perd l'historique et oblige à migrer deux fois.
- **Une seule crate** : mélange le cœur et les dépendances serveur (axum, tokio), gênant pour une compilation WASM.
