# 🛠️ Guide Pratique : Gestion Multi-Projets et Scanner

**Dernière mise à jour** : 2026-09-25  
**Public cible** : Développeurs, Utilisateurs de WikiLLM

---

## 🎯 Objectif
Scanner un répertoire contenant plusieurs dépôts de projets et basculer instantanément d'un wiki à l'autre via l'interface web ou l'API.

---

## 🔍 1. Scanner un répertoire de projets

1. Dans l'en-tête de l'interface web, ouvrez le panneau **Sélecteur de Projets**.
2. Renseignez le chemin du dossier racine contenant vos projets (ex: `~/projets`).
3. Cliquez sur **Scanner**. WikiLLM analyse tous les sous-dossiers et identifie ceux qui possèdent un dossier `wiki/`.
4. Si un dossier ne possède pas encore de wiki, le badge indique `0 notes` et vous permet de l'initialiser en un clic.

---

## 📌 2. Activer et enregistrer un projet

Lorsque vous cliquez sur un projet scanné :
- Une requête POST est émise sur `/api/activer_projet`.
- L'arborescence standard (`wiki/architecture`, `wiki/decisions`, `wiki/sessions`, `wiki/taches`) est générée si nécessaire.
- Le projet est enregistré dans `.wikillm_projets.json` pour être mémorisé lors des prochains redémarrages.

---

## 💻 3. Passer un projet en ligne de commande

Vous pouvez également enregistrer directement un projet au lancement du script :

```bash
python3 serveur.py /chemin/vers/mon-autre-projet
```

---

## 🔗 Liens utiles
- [Spécification API REST](../reference/api.md)
- [Configuration & CLI](../reference/configuration.md)
