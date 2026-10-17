# 🧠 wikik

**La doc vit dans le code ; l'agent l'interroge au lieu de relire le projet ; un wiki local la rend lisible.**

`wikik` est un binaire Rust unique qui :

- **indexe** la doc écrite dans le code (Rust `///` `//!` ; TypeScript, JavaScript et Svelte en JSDoc `/** */`) et la zone Projet (`docs/projet/` : besoins, exigences, ADR, specs) avec tree-sitter et SQLite ;
- **répond à l'agent de code** à faible coût en tokens (`wikik query …`) : sur ATS (Rust) le contexte d'une modification coûte 20 à 80 fois moins que la lecture des fichiers ([MESURE-01](./docs/projet/mesures/mesure-01-v1-agent.md)), sur shelley (Angular) un composant de 3 400 lignes se résume en 1 500 tokens ([MESURE-02](./docs/projet/mesures/mesure-02-v2-typescript.md)) ;
- **détecte la doc périmée** : le code d'une fonction a changé depuis le dernier commit qui a touché son bloc (`wikik check`) ;
- **sert un wiki local** en lecture seule : code et doc en cellules, documents du projet, liens entre les deux, graphe (`wikik serve`).

---

## 🚀 Démarrage

```bash
make install            # compile le front puis installe `wikik` dans ~/.cargo/bin (Rust + Node requis)

cd mon-projet
wikik init              # wikik.toml, docs/projet/, gabarits dans docs/modeles/, .wikik/ dans .gitignore
wikik query carte       # vue d'ensemble pour l'agent
wikik check             # doc périmée, manquante, références cassées
wikik serve             # wiki sur http://127.0.0.1:8484
```

| Commande | Rôle |
|---|---|
| `wikik query carte` · `fichier <chemin>` · `symbole <id> [--code]` · `impact <id>` | Contexte ciblé pour l'agent, coût annoncé en tokens |
| `wikik query cherche <mots>` · `adr <n>` · `exigence <EF-xxx>` · `perimes` · `ref <@wiki:…>` | Recherche et traçabilité |
| `wikik check [--format json\|github] [--strict]` | Diagnostics, codes de retour 0 / 1 / 2 |
| `wikik relu <id>` | Déclare un bloc relu après un changement de code |
| `wikik index [--dump-json]` | Index explicite (les autres commandes le mettent à jour seules) |
| `wikik export <dossier>` | Site statique du wiki (publiable tel quel, ex. GitHub Pages) |
| `wikik init --ci` · `--hook` | Workflow GitHub (check annoté, wiki publié), `CODEOWNERS`, hook pre-commit |
| `wikik nouveau <exigence\|besoin\|adr\|spec> "titre"` | Crée le prochain identifiant libre |
| `wikik stats` | Couverture de la doc |
| `wikik mcp` | Serveur MCP (stdio) : les requêtes comme outils pour l'agent. Claude Code : `claude mcp add wikik -- wikik mcp` |

Pour l'agent de code : le skill [`skill/SKILL.md`](./skill/SKILL.md) (règles d'or et procédures). Il est lié pour ce dépôt dans `.claude/skills/wikik` ; pour tous les projets : `ln -s "$PWD/skill" ~/.claude/skills/wikik`.

---

## 🧭 Documentation du projet

- 🗺️ [Plan de la v1](./docs/projet/plan-v1.md) et son tableau de suivi
- 📋 [Besoin (BDR)](./docs/projet/majeures/v1/bdr-2-definition.md) · [Cahier des charges](./docs/projet/majeures/v1/cahier-des-charges.md)
- 📐 [Specs](./docs/projet/specs/) · ⚖️ [Décisions (ADR)](./docs/projet/adr/)
- 🔭 [Suite du projet (backlog)](./docs/projet/backlog.md)
- 🤖 [Consignes pour Claude Code](./CLAUDE.md)

Le mieux reste de lancer `wikik serve` à la racine de ce dépôt : wikik se documente avec lui-même.

---

## 📁 Organisation

```text
├── crates/wikik-core/   # bibliothèque : extraction, index, check, requêtes, données du wiki
├── crates/wikik/        # binaire : CLI et serveur (front embarqué)
├── front/               # wiki : Svelte 5 + TypeScript, types générés depuis Rust
├── skill/               # SKILL.md et procédures pour l'agent
├── templates/           # gabarits de la zone Projet (embarqués par `wikik init`)
├── tests/fixtures/      # projet Rust de test et son instantané d'index
└── docs/projet/         # zone Projet de wikik lui-même (la v0 Python est au tag `v0-python`)
```
