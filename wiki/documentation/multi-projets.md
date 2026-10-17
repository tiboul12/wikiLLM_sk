# 🛠️ Guide : Gestion Multi-Projets et Scanner

**Dernière mise à jour** : 2026-09-28  
**Public cible** : Développeurs, Utilisateurs de WikiLLM

---

## 🎯 Objectif
Détecter automatiquement plusieurs projets possédant un dossier `wiki/` et basculer instantanément d'un espace de travail à un autre.

---

## 🔍 1. Scanner un répertoire de projets

1. Dans la barre latérale, cliquez sur le bouton **🔍 Scanner**.
2. Renseignez le chemin absolu du dossier parent (ex: `/home/tibal12/projets` ou `~/projets`).
3. Cliquez sur **Scanner** : le serveur analyse les sous-dossiers et affiche la liste des projets avec le décompte de leurs notes.
4. Cliquez sur un projet scanné pour l'activer immédiatement.

---

## 📌 2. Enregistrement & Persistance

- Dès qu'un projet est sélectionné, il est automatiquement enregistré dans le fichier local `.wikillm_projets.json`.
- Si le dossier ne contient pas encore de dossier `wiki/`, le serveur initialise l'arborescence standard (`architecture`, `documentation`, `decisions`, `sessions`, `taches`).

---

## 💻 3. Passer un projet en argument CLI

Vous pouvez démarrer directement sur un projet spécifique :
```bash
python3 serveur.py /chemin/vers/mon-autre-projet
```

---

## 🔗 Liens Utiles
- [[documentation/guide|Guide Général]]
- [[documentation/api-et-configuration|Référence API REST]]
- [[index|Accueil du Wiki]]
