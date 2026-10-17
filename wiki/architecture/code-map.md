# 🗺️ Cartographie Complète du Code & Blocs Doc-as-Code

Ce document fournit une **vue synthétique exhaustive en une seule passe** de l'ensemble du projet.
Il permet à tout développeur ou agent IA d'inspecter l'arborescence, les responsabilités, les classes, fonctions et blocs ADR in-code sans avoir à ouvrir chaque fichier source.

---

## 🌳 Arborescence Globale du Code

```text
📁 wikiK/
├── 📁 docs/
│   ├── 📁 assets/
│   ├── 📁 explanation/
│   │   └── 📝 architecture.md
│   ├── 📁 how-to/
│   │   ├── 📝 guide.md
│   │   ├── 📝 multi-projets.md
│   │   └── 📝 redaction-et-liens.md
│   ├── 📁 reference/
│   │   ├── 📝 api.md
│   │   └── 📝 configuration.md
│   ├── 📁 tutorials/
│   │   └── 📝 quickstart.md
│   └── 📝 index.md
├── 📁 wiki/
│   ├── 📁 architecture/
│   │   ├── 📝 code-map.md
│   │   └── 📝 vue-ensemble.md
│   ├── 📁 decisions/
│   │   └── 📝 001-choix-technologiques.md
│   ├── 📁 documentation/
│   │   ├── 📝 api-et-configuration.md
│   │   ├── 📝 guide.md
│   │   ├── 📝 multi-projets.md
│   │   └── 📝 redaction-et-liens.md
│   ├── 📁 sessions/
│   │   ├── 📝 2026-09-21-initialisation.md
│   │   ├── 📝 2026-09-25-doc-as-code.md
│   │   └── 📝 2026-09-28-unification-doc-as-code.md
│   ├── 📁 taches/
│   │   └── 📝 backlog.md
│   └── 📝 index.md
├── 🌐 index.html
├── 📝 README.md
└── 🐍 serveur.py
```

---

## 📊 Synthèse Rapide des Modules

| Fichier / Module | Type | Responsabilité | Classes | Fonctions | ADRs In-Code |
| :--- | :--- | :--- | :--- | :--- | :--- |
| [`index.html`](#-fichier-index-html) | `html/js` | Interface Web Single-Page Application (SPA) : Wiki de Projet | 0 | 29 | 0 |
| [`serveur.py`](#-fichier-serveur-py) | `python` | Serveur Web Universel Multi-Projets pour Wiki de Projet (LLM-Wiki). | 1 | 7 | 4 |

---

## 🔍 Détail Fichier par Fichier (Extraction Doc-as-Code)

### 📄 Fichier `index.html` {#-fichier-index-html}

**Description / Docstring Module :**
```text
Interface Web Single-Page Application (SPA) : Wiki de Projet
```

#### ⚡ Fonctions (29)
- **`def analyserUrlInitiale()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def mettreAJourUrl(projNom, noteChemin = null)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def chargerListeProjets(projetPrefere = null)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def changerProjet(nomProjet, noteASelectionner = null)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def chargerProjetCourant(noteASelectionner = null)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def chargerFichiers(cheminASelectionner = null)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def afficherArborescence(fichiers)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def filtrerFichiers(recherche)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def ouvrirFichier(chemin, mettreUrl = true)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def rendreMarkdown(texte)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def injecterHooks(container)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def copierHook(ancreId, elementCible)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def verifierHashUrl()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def afficherToast(messageHtml)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def trouverNote(cibleBrute)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def clicSurLienWiki(cible)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def creerPageRapide(nomFichier)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def basculerMode(mode)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def sauvegarderFichierActuel()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def supprimerFichierActuel()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def changerVue(vue)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def initialiserGraphe()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def ouvrirModalScanner()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def fermerModalScanner()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def lancerScan()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def activerProjetScanne(nom, chemin)`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def ouvrirModalCreation()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def fermerModalCreation()`** (ligne -)
  *Description* : Fonction JavaScript frontend
- **`def validerCreation()`** (ligne -)
  *Description* : Fonction JavaScript frontend

---

### 📄 Fichier `serveur.py` {#-fichier-serveur-py}

**Description / Docstring Module :**
```text
Serveur Web Universel Multi-Projets pour Wiki de Projet (LLM-Wiki).

Ce module fournit un serveur HTTP autonome (Single-Page Application et API REST)
permettant de visualiser, éditer, explorer et lier des notes Markdown au format Wiki.

Architecture Decision Record (ADR) intégré :
- Voir les blocs ADR intégrés dans les fonctions clés (zéro-dépendance, routage SPA, graphe de relations).
```

#### 🏛️ Décisions d'Architecture In-Code (ADR)
> **ADR In-Code #1 :**
> 🏛️ Architecture Decision (ADR)
> Statut : Accepté
> Contexte : Nécessité de détecter automatiquement les projets existants sans configuration manuelle.
> Décision : Parcours récursif limité (profondeur 1-2) avec détection du sous-dossier 'wiki/'.
> Conséquences & Trade-offs :
> - 🟢 Avantages : Découverte instantanée, pas de dépendance de base de données.
> - 🔴 Limites : Ne détecte pas les arborescences arbitrairement profondes au-delà de 2 niveaux.

> **ADR In-Code #2 :**
> 🏛️ Architecture Decision (ADR)
> Statut : Accepté
> Contexte : Fournir une expérience de Single Page Application (SPA) avec des URLs propres.
> Décision : Rediriger toutes les routes sans extension de fichier vers `index.html`.
> Conséquences & Trade-offs :
> - 🟢 Avantages : Navigation fluide sans rechargement, URLs de type /projet/note supportées.
> - 🔴 Limites : Nécessite que le frontend gère le routage par hash ou History API.

> **ADR In-Code #3 :**
> 🏛️ Architecture Decision (ADR)
> Statut : Accepté
> Contexte : Autoriser l'écriture et la suppression de fichiers tout en empêchant les vulnérabilités de Directory Traversal (ex: '../../etc/passwd').
> Décision : Résolution absolue via `os.path.abspath` et validation stricte du préfixe avec `os.path.abspath(dossier_wiki)`.
> Conséquences & Trade-offs :
> - 🟢 Avantages : Confinement strict et sécurisé des écritures dans le dossier wiki du projet actif.
> - 🔴 Limites : Empêche la modification de fichiers hors du dossier wiki par conception.

> **ADR In-Code #4 :**
> 🏛️ Architecture Decision (ADR)
> Statut : Accepté
> Contexte : Visualiser les interconnexions de la pensée sous forme de graphe relationnel.
> Décision : Extraction regex des syntaxes `[[note]]`, `[[note|titre]]` et `[titre](note.md)`.
> Conséquences & Trade-offs :
> - 🟢 Avantages : Analyse statique ultra-rapide sans AST Markdown lourd.
> - 🔴 Limites : Ne vérifie pas la validité sémantique fine des blocs de code échappés.

#### 🏛️ Classes
- **`class WikiHandler`** (ligne 206) : Gestionnaire HTTP personnalisé pour l'API REST et le service de l'application Web SPA.
  - `def __init__(self)` : Initialise le handler en ciblant le répertoire racine du script pour les assets statiques.
  - `def obtenir_projet_actif(self, nom_projet_ou_chemin)` : Résout le projet actif selon le nom fourni, un chemin direct ou le projet par défaut.
  - `def do_GET(self)` : Traite les requêtes HTTP GET pour l'API REST et le routage de l'interface SPA.
  - `def do_POST(self)` : Traite les requêtes HTTP POST (sauvegarde, suppression, activation de projet).
  - `def lire_corps_json(self)` : Lit et désérialise le corps d'une requête HTTP en JSON.
  - `def send_json(self, data)` : Envoie une réponse HTTP 200 avec en-têtes JSON encodés en UTF-8.
  - `def lister_fichiers(self, dossier_wiki)` : Liste tous les fichiers Markdown d'un dossier wiki avec leurs métadonnées.
  - `def generer_graphe(self, dossier_wiki)` : Extrait les relations bidirectionnelles entre notes via les liens [[wikilink]] et [label](path.md).

#### ⚡ Fonctions (7)
- **`def charger_projets_enregistres() -> Dict[str, Dict[str, str]]`** (ligne 32)
  *Description* : Charge le registre des projets sauvegardés depuis le fichier de configuration JSON.
- **`def sauvegarder_projets_enregistres(projets) -> None`** (ligne 51)
  *Description* : Sauvegarde le registre des projets dans le fichier de configuration JSON.
- **`def normaliser_chemin(chemin_brut) -> str`** (ligne 71)
  *Description* : Normalise un chemin de fichier pour Linux, macOS et Windows.
- **`def scanner_dossier_racine(chemin_racine) -> Dict[str, Any]`** (ligne 90)
  *Description* : Scanne un répertoire racine pour détecter les projets et sous-projets contenant un dossier wiki.
- **`def compter_notes(dossier_wiki) -> int`** (ligne 147)
  *Description* : Compte récursivement le nombre total de notes Markdown (`.md`) dans un répertoire wiki.
- **`def initialiser_wiki_projet(chemin_projet) -> str`** (ligne 164)
  *Description* : Initialise l'arborescence standard d'un wiki au sein d'un répertoire de projet.
- **`def demarrer(port) -> None`** (ligne 541)
  *Description* : Démarre le serveur HTTP multi-projets et écoute sur l'ensemble des interfaces réseau.

---

## 🔗 Liens Utiles
- [[architecture/vue-ensemble|Architecture & Spécifications]]
- [[decisions/001-choix-technologiques|Registre des Décisions (ADR)]]
- [[documentation/api-et-configuration|Référence API & Configuration]]
- [[taches/backlog|Backlog & Roadmap]]
