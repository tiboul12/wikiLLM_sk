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
- [ ] **Activer la CI et Pages sur wikiK** : `wikik init --ci`, puis activer GitHub Pages (source : GitHub Actions) ; non fait automatiquement car cela publie le wiki.
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
- [x] **Workflow CI** : `wikik init --ci` (v3, EF-082).
- [x] **Publication du wiki en site statique** : `wikik export` (v3, EF-080, ADR-0008).
- [x] **Rôles de validation** : `[equipe] validateurs` → `CODEOWNERS` (v3, EF-082). Activer « Require review from Code Owners » sur GitHub.
- [x] **Identifiants** : `wikik nouveau` + `id-duplique` bloquant en CI (v3, EF-084).
- [x] **Annotations GitHub** : `check --format github` (v3, EF-081).
- [x] **Métriques** : `wikik stats` (v3, EF-085). Tendance dans le temps : à faire (artefact de CI).
- [x] **Hook pre-commit** : `wikik init --hook` (v3, EF-083).
- [x] **Niveaux de sévérité configurables** : `[check.severites]` (v1).
- [ ] **Backlog dans un outil de tickets** (GitHub Issues) lu par le wiki (Q1).
- [ ] **Portail multi-dépôts**, héritier du registre multi-projets (Q7).

### 🌐 Front & WebAssembly (A3)
- [~] **Moteur de requêtes en WASM** : remplacé par des données pré-calculées (ADR-0008) ; à reprendre seulement pour de très gros projets.
- [ ] **Responsive mobile** (tiroir latéral, barre d'actions adaptée).
- [ ] **Thèmes visuels**.
- [ ] **Édition dans le navigateur** pour la zone Projet (Q5 : non en v1).

### 🗣️ Langages supplémentaires (A4)
- [x] **TypeScript** : v2 (EF-070 à EF-075, SPEC-15, MESURE-02 sur shelley-studio-llm).
- [x] **JavaScript** (`.js`, `.jsx`, `.mjs`, `.cjs`) avec la grammaire TSX : v2.1 (EF-076).
- [x] **Svelte** : script extrait, composant comme symbole, `<Composant>` rendus : v2.1 (EF-077).
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
