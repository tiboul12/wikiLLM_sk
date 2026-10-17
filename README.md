# 🧠 WikiLLM (wikiK) - Documentation Vivante Multi-Projets

Un système universel de documentation vivante pour vos projets, éditable de manière collaborative par vous et vos assistants IA (LLMs), consultable instantanément sur votre réseau local via votre navigateur web.

---

## 🚀 Démarrage Rapide

Ce projet ne nécessite **aucune installation de bibliothèque** (utilise uniquement Python 3 standard).

Lancez simplement le serveur depuis la racine du projet :

```bash
python3 serveur.py
```

Ouvrez ensuite votre navigateur sur : **[http://localhost:8000](http://localhost:8000)** (ou sur l'IP réseau affichée pour consultation mobile/tablette).

---

## 🧭 Documentation Technique (Diátaxis)

La documentation complète du projet est organisée selon la méthodologie **Docs-as-Code / Diátaxis** dans le dossier [`docs/`](./docs/index.md) :

- 🎓 **[Tutoriel de Prise en Main](./docs/tutorials/quickstart.md)** : Vos premiers pas en 5 minutes.
- 🛠️ **[Guides Pratiques](./docs/how-to/multi-projets.md)** :
  - [Scanner et gérer le multi-projets](./docs/how-to/multi-projets.md)
  - [Rédaction de notes et syntaxe des wikilinks](./docs/how-to/redaction-et-liens.md)
- 🔌 **[Référence](./docs/reference/api.md)** :
  - [Spécification complète de l'API REST](./docs/reference/api.md)
  - [Configuration et Ligne de commande CLI](./docs/reference/configuration.md)
- 🏛️ **[Architecture & ADRs](./docs/explanation/architecture.md)** :
  - Architecture détaillée, diagrammes Mermaid et registre de décisions intégré.

---

## 📁 Organisation des Fichiers

```text
├── index.html               # Interface web monopage (Viewer Markdown, Éditeur Ctrl+S, Graphe de liens)
├── serveur.py               # Serveur Web & API REST Python natif (zéro dépendance pip)
├── docs/                    # Documentation technique Diátaxis (Docs-as-Code)
│   ├── tutorials/           # Tutoriels d'apprentissage
│   ├── how-to/              # Guides pratiques et recettes
│   ├── reference/           # Spécifications API & configuration
│   └── explanation/         # Architecture et décisions (ADR)
├── wiki/                    # Contenu documentaire vivant du projet
│   ├── index.md             # Sommaire du wiki
│   ├── architecture/        # Spécifications fonctionnelles
│   ├── decisions/           # Fiches ADR
│   ├── sessions/            # Journal des séances de travail
│   └── taches/              # Roadmap et backlog
└── README.md                # Point d'entrée du dépôt
```

---

## 🔗 Liens entre pages

Dans vos notes, liez vos documents facilement en écrivant `[[NomDeLaPage]]` ou `[[dossier/NomDeLaPage]]`. Le serveur analyse automatiquement les relations pour construire la cartographie interactive dans l'onglet **Graphe**.
