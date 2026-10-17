# 🎓 Tutoriel : Prise en main rapide de WikiLLM

**Durée estimée** : 5 minutes  
**Dernière mise à jour** : 2026-09-25  
**Prérequis** : Python 3.8+ installé.

---

## 🎯 Objectif
Dans ce tutoriel, vous allez lancer le serveur WikiLLM, explorer l'interface web, créer une note Markdown et observer le graphe de relations interactif.

---

## 🚀 Étape 1 : Démarrage du serveur

Depuis la racine du projet `wikiK`, exécutez la commande Python native :

```bash
python3 serveur.py
```

Le terminal affiche les URLs d'accès local et réseau :
```text
====================================================================
 🚀 WikiLLM Serveur Multi-Projets Démarré !
 📂 Projets enregistrés : wikiK
--------------------------------------------------------------------
 🌐 Accès Hub      : http://localhost:8000/
 🌐 Accès Réseau   : http://10.0.0.120:8000/
 🌐 Exemple Projet : http://10.0.0.120:8000/wikiK/index
====================================================================
```

---

## 🌐 Étape 2 : Consultation de l'interface Web

1. Ouvrez votre navigateur sur **[http://localhost:8000](http://localhost:8000)**.
2. L'arborescence des fichiers du dossier `wiki/` apparaît sur la barre latérale gauche.
3. Cliquez sur `architecture/vue-ensemble` pour charger la note dans le visualiseur Markdown.

---

## ✍️ Étape 3 : Créer et éditer une note

1. Dans la barre latérale ou via le bouton **Nouvelle Note**, saisissez le chemin `taches/roadmap`.
2. Tapez le contenu suivant en intégrant un lien Wiki :
   ```markdown
   # 🗺️ Roadmap du Projet
   
   Consultez notre [[architecture/vue-ensemble|Architecture]] pour plus de détails.
   ```
3. Sauvegardez avec `Ctrl+S` (ou le bouton **Sauvegarder**). Le fichier physique `wiki/taches/roadmap.md` est instantanément créé sur votre disque.

---

## 🕸️ Étape 4 : Visualiser le graphe de connaissances

1. Dans le menu de navigation supérieur de l'interface, cliquez sur l'onglet **Graphe**.
2. Une vue interactive en réseau apparaît, reliant votre nouvelle note `taches/roadmap` à `architecture/vue-ensemble`.
3. Cliquez sur un nœud pour ouvrir directement le document correspondant.

---

## 🎉 Félicitations !
Vous maîtrisez désormais le cycle de base de WikiLLM. Pour aller plus loin :
- [Guide Multi-Projets](../how-to/multi-projets.md)
- [Rédaction et liens](../how-to/redaction-et-liens.md)
- [Architecture du système](../explanation/architecture.md)
