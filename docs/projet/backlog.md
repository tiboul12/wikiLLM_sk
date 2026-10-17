---
id: BACKLOG
type: backlog
titre: "Backlog et suite du projet"
statut: vivant
date: 2026-10-17
sources: [PLAN-v1]
---

# 📋 Backlog & Feuille de Route

**Dernière mise à jour** : 2026-10-17
**Source des décisions** : `.doc/reflexion et evolution/analyse-2026-10-17.md` (décisions A1 à A6) et [[plan-v1]]

---

## 🧭 Cap actuel (17/10/2026)

Refonte de wikiK en outil **Rust** (un seul binaire) : extracteur tree-sitter, index SQLite, `check`, CLI de requêtes pour l'agent, puis wiki généré avec un front TypeScript. v1 **solo et en local**, architecture prête pour l'équipe. Plan détaillé de la v1 : [[plan-v1]] (phases 0 à 9).

---

## ✋ À faire avec Tibo pour passer de `v1.0-rc1` à `v1.0`

- [ ] **Relire et valider** la BDR, le cahier des charges, les specs et les ADR produits par délégation le 17/10 (statut `proposé`), et corriger ce qui ne convient pas.
- [ ] **Mesure en session réelle** sur ATS (protocole dans MESURE-01).
- [ ] **Installer** : `make install`, puis le skill pour tous les projets (`ln -s "$PWD/skill" ~/.claude/skills/wikik`) en remplacement de `doc-as-code` et `wiki-dev-sync`.
- [ ] **Amorcer ATS** (procédure `skill/procedures/amorcer.md`) : `wikik init`, puis les blocs manquants module par module ; migrer `ATS/wiki/` et le cahier des charges d'ATS vers `docs/projet/` (convertir `AC-009`, `F1`… en `EF-xxx`).
- [ ] **Trancher Leptos ou Svelte** pour le front (SPEC-05) : Svelte a été choisi par défaut.

## 🧩 Constats de la v1 à traiter

- [x] **Indexer le front** (TypeScript, v2) : fait, EF-045 est citée par `front/src/lib/hook.ts` ; les `.svelte` restent à faire.
- [ ] **`check` sur un gros projet** : un `git blame` par fichier documenté ; à mesurer au-delà de 50 000 lignes, cache déjà en place.
- [ ] **Appels de méthode** : seuls les noms uniques sont résolus (`probable`) ; envisager une résolution par type des champs `self.x.f()`.

## 🗄️ Reporté après la v1 (à reprendre pour la suite du projet)

Tout ce qui a été volontairement laissé de côté pendant l'analyse du 17/10. Rien ici n'est abandonné.

### 👥 Passage en équipe (A1 : v1 solo, architecture prête)
- [ ] **Workflow CI** : build de l'index, `check` sur chaque PR, génération et publication du site.
- [ ] **Publication du wiki en site statique** (GitHub Pages ou équivalent interne).
- [ ] **Rôles de validation** BDR / cahier des charges dans `doc.toml`, approbation obligatoire sur PR (Q10).
- [ ] **Attribution d'identifiants sans collision** entre PR parallèles (ADR, `B-xxx`, `EF-xxx`).
- [ ] **Annotations GitHub** depuis la sortie JSON de `check`.
- [ ] **Métriques partagées** : couverture des blocs, nombre de périmés, tendance.
- [ ] **Hook pre-commit** optionnel.
- [ ] **Niveaux de sévérité configurables** (avertissement puis bloquant).
- [ ] **Backlog dans un outil de tickets** (GitHub Issues) lu par le wiki (Q1).
- [ ] **Portail multi-dépôts**, héritier du registre multi-projets (Q7).

### 🌐 Front & WebAssembly (A3)
- [ ] **Moteur de requêtes compilé en WASM** pour le wiki statique publié : recherche et graphe dans le navigateur, sans serveur.
- [ ] **Responsive mobile** (tiroir latéral, barre d'actions adaptée).
- [ ] **Thèmes visuels**.
- [ ] **Édition dans le navigateur** pour la zone Projet (Q5 : non en v1).

### 🗣️ Langages supplémentaires (A4)
- [x] **TypeScript** : v2 (EF-070 à EF-075, SPEC-15, MESURE-02 sur shelley-studio-llm).
- [ ] **JavaScript** (`.js`, `.jsx`, `.mjs`) avec la grammaire TSX.
- [ ] **Svelte** : extraire la partie `<script lang="ts">` des `.svelte` (front de wikik).
- [ ] **Gabarits Angular** : appels de méthodes depuis les templates HTML (heuristique, comme `view!` en Rust).
- [ ] **Alias `tsconfig`** (`paths`) pour résoudre `@scope/paquet` vers les fichiers du dépôt.
- [ ] **Python**.

### 🤖 Interface agent
- [x] **Serveur MCP** exposant les mêmes requêtes que la CLI : `wikik mcp`, v1.1 (EF-060, SPEC-14).
- [ ] **RAG structuré** (SPEC-08) : embeddings des blocs (local ou API, Q8), top-k sémantique puis expansion dans le graphe, jeu d'évaluation de 30 questions. Seulement après une v1 utilisée.

### 🎨 Atelier de conception Fast Design Thinking (A5 : hors v1)
- [ ] **Réécrire le framework en spécifications déclaratives** : un fichier par technique (entrées, sorties, prompts, schéma d'artefact, règles de validation) au lieu du méga-prompt de 216 Ko.
- [ ] **Procédure conversationnelle « ouvrir une mise à jour majeure »** en mode Express, brouillons dans `.doc/atelier/vN/`, puis BDR et cahier des charges.
- [ ] **Étape 0 « Pitch »** (problème, cible, valeur, contraintes, critères de succès, hors périmètre).
- [ ] **Atelier dans l'UI, mode local uniquement**, avec appels LLM par étape (contexte limité aux pages sources).
- [ ] **Écrans dédiés** : free writing, mind map, grille SCAMPER, storyboards R.E.C.I.T., tri des besoins.
- [ ] **Vue « Parcours »** : stepper des phases, compteurs, pages en attente de validation.
- [ ] **Modes Express et Complet**, variantes générées en lot et validées par sélection.

### 📐 Cycle des mises à jour majeures
- [ ] **Vue consolidée du cahier des charges** générée en rejouant les versions (Q9).
- [ ] **Définition d'une « majeure »** (Q11 ; défaut : ajout/retrait d'une exigence `must` ou contrat public cassé).
- [ ] **Matrice de traçabilité** besoin → exigence → spec/ADR → symboles → tests, par version.

### 🔁 Migration (SPEC-12)
- [ ] **Archiver les anciens skills** `doc-as-code` et `wiki-dev-sync`, en reprenant leurs conventions utiles.
- [ ] **Migrer les wikis existants** (`wiki/` de wikiK et d'ATS) vers la nouvelle structure `docs/projet/`.
- [ ] **Sort du journal `sessions/`** (Q4 ; défaut : remplacé par les PR et les ADR).

---

## 🚀 Historique : wikiK v0 (Python, avant la refonte)

- [x] **Socle Serveur Léger** : Serveur HTTP natif en Python 3 sans dépendance pip (`serveur.py`).
- [x] **Interface Web Monopage (SPA)** : Visualiseur Markdown, éditeur avec sauvegarde (`Ctrl+S`), Hooks Agent fiabilisés.
- [x] **Navigation Épurée & Réductible** : Suppression de la catégorie racine, accès rapide Index/Backlog, catégories repliables.
- [x] **Scanner Multi-Projets** : Détection automatique des répertoires de projets avec dossier `wiki/`.
- [x] **Moteur de Graphe** : Visualisation interactive des liens `[[wikilink]]` avec Vis.js.
- [x] **Cartographie AST en 1 Passe** : Vue globale unifiée du code, fonctions, classes et blocs ADR dans `wiki/architecture/code-map.md`.
- [ ] **Historique & Intégration Git** : fonctions écrites dans `serveur.py` (`obtenir_git_statut/historique/diff`) mais **branchées sur aucune route** au 17/10. Non porté tel quel.

---

## 📌 Notes Architecturales & Décisions Futures

> 💡 **Recherche plein texte** : reprise par l'index (SQLite FTS5) de la refonte.
>
> 💡 **Export statique** : écarté en septembre, **réintroduit** pour le passage en équipe (site statique publié par la CI, voir plus haut).
