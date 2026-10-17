# 🔌 Spécification de l'API REST de WikiLLM

**Dernière mise à jour** : 2026-09-25  
**Public cible** : Développeurs, Intégrateurs  
**Base URL** : `http://localhost:8000` (ou IP réseau locale)

---

<!-- Embedded Architecture Decision Record (ADR Block) -->
> ### 🏛️ Architecture Decision (ADR)
> **Statut** : Accepté  
> **Contexte** : Fournir une interface de données synchrone, légère et manipulable par JavaScript ou des scripts externes.  
> **Décision** : API HTTP/JSON native sans framework (FastAPI/Flask) pour conserver une portabilité totale avec Python standard.  
> **Conséquences & Trade-offs** :
> - 🟢 **Avantages** : Zéro dépendance, temps de démarrage instantané.
> - 🔴 **Limites** : Validation des schémas réalisée manuellement dans le serveur.

---

## 📋 Sommaire des Endpoints

| Méthode | Route | Description |
| :--- | :--- | :--- |
| `GET` | `/api/scanner` | Scanne un dossier racine et détecte les wikis de projets. |
| `GET` | `/api/projets` | Retourne la liste des projets enregistrés. |
| `GET` | `/api/projet_info` | Retourne les détails du projet actif sélectionné. |
| `GET` | `/api/fichiers` | Liste les fichiers Markdown du projet actif. |
| `GET` | `/api/fichier` | Récupère le contenu d'un fichier Markdown donné. |
| `GET` | `/api/graphe` | Génère le graphe relationnel des notes (nœuds et liens). |
| `POST` | `/api/activer_projet` | Active et initialise un projet à partir d'un chemin de répertoire. |
| `POST` | `/api/sauvegarder` | Sauvegarde ou crée une note Markdown sur le disque. |
| `POST` | `/api/supprimer` | Supprime un fichier Markdown du projet actif. |

---

## 🔍 `GET /api/scanner`

### Paramètres de requête
| Nom | Type | Obligatoire | Description |
| :--- | :--- | :--- | :--- |
| `dossier` | `string` | Non | Chemin racine à scanner (ex: `~/projets`). Par défaut dossier courant. |

### Réponse (200 OK)
```json
{
  "erreur": null,
  "racine": "/home/user/projets",
  "projets": [
    {
      "id": "wikiK",
      "nom": "wikiK",
      "chemin": "/home/user/projets/wikiK",
      "a_wiki": true,
      "nb_notes": 8
    }
  ]
}
```

---

## 📁 `GET /api/projets`

### Réponse (200 OK)
```json
{
  "projets": [
    {
      "nom": "wikiK",
      "chemin": "/home/user/projets/wikiK"
    }
  ]
}
```

---

## ℹ️ `GET /api/projet_info`

### Paramètres de requête
| Nom | Type | Obligatoire | Description |
| :--- | :--- | :--- | :--- |
| `projet` | `string` | Non | Nom ou identifiant du projet ciblé. |

### Réponse (200 OK)
```json
{
  "nom_projet": "wikiK",
  "repertoire": "/home/user/projets/wikiK",
  "dossier_wiki": "/home/user/projets/wikiK/wiki",
  "nb_notes": 8
}
```

---

## 📑 `GET /api/fichiers`

### Paramètres de requête
| Nom | Type | Obligatoire | Description |
| :--- | :--- | :--- | :--- |
| `projet` | `string` | Non | Nom du projet cible. |

### Réponse (200 OK)
```json
[
  {
    "nom": "vue-ensemble.md",
    "chemin": "architecture/vue-ensemble.md",
    "dossier": "architecture",
    "taille": 1296
  }
]
```

---

## 📖 `GET /api/fichier`

### Paramètres de requête
| Nom | Type | Obligatoire | Description |
| :--- | :--- | :--- | :--- |
| `chemin` | `string` | Oui | Chemin relatif de la note (ex: `architecture/vue-ensemble.md`). |
| `projet` | `string` | Non | Nom du projet cible. |

### Réponse (200 OK)
```json
{
  "chemin": "architecture/vue-ensemble.md",
  "contenu": "# 🏗️ Architecture & Vue d'ensemble..."
}
```

---

## 🕸️ `GET /api/graphe`

### Paramètres de requête
| Nom | Type | Obligatoire | Description |
| :--- | :--- | :--- | :--- |
| `projet` | `string` | Non | Nom du projet cible. |

### Réponse (200 OK)
```json
{
  "nodes": [
    { "id": "index.md", "label": "index", "group": "Racine" },
    { "id": "architecture/vue-ensemble.md", "label": "vue-ensemble", "group": "architecture" }
  ],
  "edges": [
    { "from": "index.md", "to": "architecture/vue-ensemble.md" }
  ]
}
```

---

## ➕ `POST /api/activer_projet`

### Body (JSON)
```json
{
  "chemin": "/home/user/projets/nouveau-projet",
  "nom": "nouveau-projet"
}
```

### Réponse (200 OK)
```json
{
  "statut": "succes",
  "projet": {
    "nom": "nouveau-projet",
    "chemin": "/home/user/projets/nouveau-projet"
  }
}
```

---

## 💾 `POST /api/sauvegarder`

### Body (JSON)
```json
{
  "projet": "wikiK",
  "chemin": "taches/roadmap.md",
  "contenu": "# Roadmap\n- [ ] Nouvelle tâche"
}
```

### Réponse (200 OK)
```json
{
  "statut": "succes",
  "chemin": "taches/roadmap.md"
}
```

---

## 🗑️ `POST /api/supprimer`

### Body (JSON)
```json
{
  "projet": "wikiK",
  "chemin": "taches/roadmap.md"
}
```

### Réponse (200 OK)
```json
{
  "statut": "succes"
}
```
