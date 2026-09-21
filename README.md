# 🧠 WikiLLM - Wiki de Projet Léger

Un système de documentation vivante pour vos projets, éditable par vous et par vos assistants IA, consultable directement dans votre navigateur web.

## 🚀 Démarrage Rapide

Ce projet ne nécessite **aucune installation de bibliothèque** (utilise uniquement Python 3 standard).

Lancez simplement le serveur depuis la racine du projet :

```bash
python3 serveur.py
```

Ouvrez ensuite votre navigateur sur : **[http://localhost:8000](http://localhost:8000)**

---

## 📁 Organisation des Fichiers

* `index.html` : Interface web moderne (Viewer Markdown, Éditeur avec raccourci Ctrl+S, Graphe de relations, Recherche instantanée).
* `serveur.py` : Serveur web Python léger (gère la lecture, l'écriture et l'analyse des liens).
* `wiki/` : Répertoire contenant toutes vos notes en Markdown pur :
  * `wiki/index.md` : Page d'accueil et table des matières.
  * `wiki/decisions/` : Fiches de décisions d'architecture (ADR).
  * `wiki/sessions/` : Comptes-rendus des sessions de travail.
  * `wiki/architecture/` : Vues d'ensemble et spécifications.
  * `wiki/taches/` : Tâches et roadmap.

---

## 🔗 Liens entre pages

Vous pouvez lier vos pages entre elles simplement en écrivant `[[NomDeLaPage]]` ou `[[dossier/NomDeLaPage]]`. Le serveur calculera automatiquement les relations pour afficher la carte interactive dans l'onglet **Graphe**.
