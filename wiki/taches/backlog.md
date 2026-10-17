# 📋 Backlog & Feuille de Route

**Dernière mise à jour** : 2026-09-28  
**Référence** : [[index|Sommaire du Wiki]]

---

## 🚀 Fonctionnalités Terminées

- [x] **Socle Serveur Léger** : Serveur HTTP natif en Python 3 sans dépendance pip (`serveur.py`).
- [x] **Interface Web Monopage (SPA)** : Visualiseur Markdown, éditeur avec sauvegarde (`Ctrl+S`), Hooks Agent fiabilisés.
- [x] **Navigation Épurée & Réductible** : Suppression de la catégorie racine, accès rapide Index/Backlog, catégories repliables.
- [x] **Scanner Multi-Projets** : Détection automatique des répertoires de projets avec dossier `wiki/`.
- [x] **Moteur de Graphe** : Visualisation interactive des liens `[[wikilink]]` avec Vis.js.
- [x] **Cartographie AST en 1 Passe** : Vue globale unifiée du code, fonctions, classes et blocs ADR dans `wiki/architecture/code-map.md`.
- [x] **Historique & Intégration Git** : Suivi des révisions, diffs et historique de commits dans l'interface web.

---

## 🎯 Prochaines Fonctionnalités & Priorités

### 🔥 Priorité Haute / En cours
- [ ] **Recherche Plein Texte Avancée** :
  - Indexation et recherche dans le contenu intégral des notes Markdown (pas uniquement les titres et chemins).

### ⏳ Priorité Basse / Secondaire
- [ ] **Mode Smartphone / Responsive Mobile de l'UI** :
  - Optimisation responsive tactile avancée (tiroir latéral mobile, barre d'actions adaptée aux petits écrans).
- [ ] **Mode Sombre / Thèmes** :
  - Sélecteur de thèmes visuels personnalisés pour l'interface web.

---

## 📌 Notes Architecturales & Décisions Futures

> 💡 **Export & Intégration RAG** :  
> L'export PDF / HTML statique autonome a été écarté comme superflu. Si un mécanisme d'export ou d'ingestion est développé à l'avenir, il sera orienté vers un **pipeline / connecteur RAG** (Retrieval-Augmented Generation) pour alimenter des bases vectorielles ou des contextes d'agents.
