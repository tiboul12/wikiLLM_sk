# 🏗️ Architecture & Vue d'ensemble

Ce document décrit l'organisation du projet et la circulation de l'information entre l'agent et le développeur.

## 📂 Structure des Fichiers

```text
├── index.html       # Page Web de consultation et d'édition (Vue client)
├── serveur.py       # Serveur Web Python léger (zéro dépendance pip)
├── README.md        # Guide de lancement rapide
└── wiki/            # Contenu documentaire en Markdown pur
    ├── index.md            # Sommaire central
    ├── architecture/       # Spécifications et vues système
    ├── decisions/          # Choix d'architecture (ADR)
    ├── sessions/           # Comptes-rendus des sessions de travail
    └── taches/             # Suivi des tâches et roadmap
```

## 🔄 Flux de Travail
1. **Pendant le travail** : L'agent et l'utilisateur conçoivent et codent le projet.
2. **À chaque étape clé** : L'agent consigne les décisions dans `decisions/` et résume la séance dans `sessions/`.
3. **Dans le navigateur** : L'utilisateur lance `python3 serveur.py` pour naviguer, éditer ou voir le graphe des connexions.

---
Liens utiles :
- [[index|Accueil du Wiki]]
- [[decisions/001-choix-technologiques|Décision 001]]
- [[taches/backlog|Backlog]]
