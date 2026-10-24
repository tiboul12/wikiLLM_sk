---
id: ADR-0020
type: adr
titre: "Recherche sémantique : embeddings d'Ollama, vecteurs gardés dans .wikik/"
statut: proposé
date: 2026-10-24
exigences: [EF-173]
---

# ADR-0020 : recherche sémantique : embeddings d'Ollama, vecteurs gardés dans `.wikik/`

## Contexte

La recherche plein texte (FTS5) ne trouve que les mots écrits ; Tibo veut retrouver ce qui répond à une question en langage naturel, « comme grepai, mais en Rust » (REC-097). grepai découpe le code, calcule un vecteur par morceau avec un modèle d'embeddings (Ollama en local, ou une API), garde les vecteurs et répond par similarité cosinus ; il est fait pour les agents de code. wikik a déjà des morceaux tout faits : chaque bloc de doc et chaque document est une ligne de la table `recherche` de l'index.

## Décision

- **Morceaux** : les lignes de la table `recherche` (symboles documentés avec leur titre et leur doc, documents avec leur titre et leur texte), tronquées à 2 000 caractères. Pas de découpe du code lui-même : la doc est ce que wikik sait dire d'un symbole.
- **Embeddings** : l'API `/api/embed` d'Ollama (le serveur local déjà utilisé par le connecteur Ollama, ou celui de Shelley), appelée par `curl` comme les autres connecteurs. Modèle par défaut `nomic-embed-text`, réglable dans `[semantique]` de `wikik.toml` (`modele`, `url`). Rien ne quitte la machine (ENF-004).
- **Stockage** : `.wikik/semantique.sqlite`, non versionné et reconstructible comme l'index (ADR-0003) ; une ligne par morceau avec l'empreinte de son texte et du modèle : seuls les morceaux nouveaux ou modifiés sont recalculés, par lots de 32.
- **Requête** : le vecteur de la question, comparé par cosinus à tous les vecteurs (quelques milliers : un parcours suffit), les 20 meilleurs rendus avec leur score. Servie à la page Recherche (« par le sens ») et aux agents par l'outil MCP `cherche_sens`.
- **Facultative** : sans Ollama joignable ou sans le modèle, la recherche par le sens dit comment l'activer (`ollama pull nomic-embed-text`) ; la recherche plein texte et la recherche floue restent.

## Conséquences

- La première recherche calcule tous les vecteurs (une à quelques minutes selon la machine) ; les suivantes ne recalculent que ce qui a changé.
- Changer de modèle recalcule tout (l'empreinte inclut le modèle).

## Alternatives écartées

- **Un modèle embarqué dans le binaire** (ONNX, model2vec) : des dizaines de Mo de plus et un téléchargement de modèle au premier lancement, pour un service qu'Ollama rend déjà sur les machines de Tibo.
- **Une base vectorielle** (Qdrant, extension sqlite-vec) : inutile à cette taille, et une dépendance de plus.
- **Une API d'embeddings en ligne** : contraire à ENF-004.
