# 🧠 wikiK

> **Refonte en cours.** wikiK devient un outil Rust (un seul binaire) qui indexe la documentation écrite dans le code d'un projet, détecte la doc périmée, répond aux requêtes d'un agent de code à faible coût en tokens, puis affiche le tout dans un wiki local.

- 🗺️ **Plan de la v1** : [`docs/projet/plan-v1.md`](./docs/projet/plan-v1.md)
- 📋 **Backlog et suite du projet** : [`wiki/taches/backlog.md`](./wiki/taches/backlog.md)
- 🤖 **Consignes pour l'agent** : [`CLAUDE.md`](./CLAUDE.md)

---

## 🕰️ Version v0 (Python) : `legacy/`

La v0, un wiki Markdown multi-projets éditable dans le navigateur, reste utilisable jusqu'à la fin de la refonte. Elle est figée au tag `v0-python`.

```bash
python3 legacy/serveur.py
```

Puis ouvrir **[http://localhost:8000](http://localhost:8000)**. Aucune dépendance (Python 3 standard).

Sa documentation (structure Diátaxis) est dans [`legacy/docs/`](./legacy/docs/index.md), et son wiki de projet dans [`wiki/`](./wiki/index.md).

---

## 📁 Organisation

```text
├── CLAUDE.md          # Consignes pour Claude Code
├── docs/projet/       # Zone Projet de la refonte : plan, BDR, exigences, specs, ADR
├── wiki/              # Wiki de la v0 (migré vers docs/projet/ en Phase 9)
└── legacy/            # v0 Python : serveur.py, index.html, docs Diátaxis
```
