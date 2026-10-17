# 🔌 Configuration & Ligne de Commande (CLI)

**Dernière mise à jour** : 2026-09-25  
**Public cible** : Administrateurs, Développeurs

---

## 💻 Arguments CLI de `serveur.py`

Le script `serveur.py` accepte des arguments positionnels facultatifs :

```bash
python3 serveur.py [PORT] [CHEMIN_PROJET_SUPPLEMENTAIRE]
```

### Exemples d'utilisation :

1. **Port par défaut (8000)** :
   ```bash
   python3 serveur.py
   ```

2. **Changer le port d'écoute** :
   ```bash
   python3 serveur.py 9000
   ```

3. **Enregistrer un projet externe au démarrage** :
   ```bash
   python3 serveur.py 8080 ~/projets/mon-autre-projet
   ```

---

## ⚙️ Fichier de persistance `.wikillm_projets.json`

À la racine du répertoire de `serveur.py`, le fichier de configuration JSON stocke le registre des projets connus :

```json
{
  "wikiK": {
    "nom": "wikiK",
    "chemin": "/home/user/projets/wikiK"
  },
  "autre-projet": {
    "nom": "autre-projet",
    "chemin": "/home/user/projets/autre-projet"
  }
}
```

Ce fichier est automatiquement mis à jour lors de l'activation d'un projet via l'interface web ou l'API.

---

## 🌐 Détection IP Réseau
Le serveur tente d'ouvrir un socket UDP sortant éphémère vers `8.8.8.8:80` pour détecter l'adresse IP réseau locale de la machine hôte et l'afficher dans les logs de démarrage. Si le réseau est indisponible, il bascule sur `127.0.0.1`.
