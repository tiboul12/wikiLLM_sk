# ⚖️ ADR 001 : Choix Technologiques du Wiki

* **Date** : 2026-09-21
* **Statut** : Validé
* **Contexte** : [[index|Retour au Sommaire]] | [[architecture/vue-ensemble|Architecture]]

## Contexte et Problématique
Le projet nécessitait une solution ultra-légère pour documenter les sessions de travail et les décisions d'architecture avec un agent LLM, consultable et éditable directement depuis un navigateur web, sans dépendance lourde ni base de données complexe.

## Décision
1. Utilisation de fichiers **Markdown purs** stockés dans le dossier `wiki/`.
2. Serveur local léger en **Python standard** (`serveur.py`) sans dépendances externes `pip`.
3. Interface Web monopage (`index.html`) à la racine avec :
   - Rendu Markdown temps réel.
   - Éditeur avec sauvegarde sur disque.
   - Graphe interactif de relations `[[liens]]`.

## Conséquences et Bénéfices
* Déploiement instantané dans n'importe quel dépôt Git.
* Compatibilité totale avec Git, Obsidian, ou n'importe quel éditeur de texte.
* Maintenance par l'agent LLM transparente et simple.
