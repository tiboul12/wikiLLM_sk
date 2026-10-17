# 🔌 Référence Technique : API REST & Configuration

**Dernière mise à jour** : 2026-09-28  
**Public cible** : Développeurs, Intégrateurs, Agents IA  
**Base URL** : `http://localhost:8000`

---

## 📋 Endpoints de l'API REST

| Méthode | Route | Description |
| :--- | :--- | :--- |
| `GET` | `/api/scanner?dossier=...` | Scanne un répertoire et liste les projets avec sous-dossier `wiki/`. |
| `GET` | `/api/projets` | Liste l'ensemble des projets mémorisés dans le registre. |
| `GET` | `/api/projet_info?projet=...` | Retourne les informations et le nombre de notes du projet actif. |
| `GET` | `/api/fichiers?projet=...` | Liste tous les fichiers `.md` avec nom, dossier parent et taille. |
| `GET` | `/api/fichier?projet=...&chemin=...` | Retourne le contenu brut d'un fichier Markdown. |
| `GET` | `/api/graphe?projet=...` | Calcule les nœuds et liens du graphe relationnel (`[[wikilinks]]`). |
| `POST` | `/api/activer_projet` | Enregistre et initialise un nouveau projet (`{ "chemin": "...", "nom": "..." }`). |
| `POST` | `/api/sauvegarder` | Sauvegarde une note sur le disque (`{ "projet": "...", "chemin": "...", "contenu": "..." }`). |
| `POST` | `/api/supprimer` | Supprime définitivement une note du projet (`{ "projet": "...", "chemin": "..." }`). |

---

## ⚙️ Configuration & Options CLI

Le serveur accepte des arguments positionnels facultatifs :
```bash
python3 serveur.py [PORT] [CHEMIN_PROJET_SUPPLEMENTAIRE]
```

- **Port personnalisé** : `python3 serveur.py 9000`
- **Ajout de projet externe** : `python3 serveur.py 8000 ~/projets/autre-projet`
- **Fichier de registre** : `.wikillm_projets.json` (persistance locale automatique).

---

## 🔗 Liens Utiles
- [[documentation/guide|Guide Général]]
- [[architecture/code-map|Cartographie du Code]]
- [[index|Accueil du Wiki]]
