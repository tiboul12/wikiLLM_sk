---
name: wikik-lecture
description: Lire un projet indexé par wikik sans ouvrir ses fichiers source. À utiliser dès qu'il faut comprendre du code, une exigence, une décision ou un document d'un projet qui a un index wikik (outils MCP carte, fichier, symbole, impact, cherche, document, exigence, adr).
---

# Lire le projet avec wikik

Ce projet a un index wikik : la doc écrite dans le code (`///`, `//!`, JSDoc) et la zone Projet (besoins, exigences, décisions, specs) sont interrogeables. Interroger l'index coûte bien moins cher que relire les fichiers.

## Méthode

1. **Début de session** : `carte` donne les fichiers du projet et le rôle de chacun.
2. **Avant de parler d'un fichier** : `fichier <chemin>` donne ses symboles et leur doc, sans le code.
3. **Avant de parler d'un symbole** : `symbole <id>` (doc, appels, exigences, décisions) puis `impact <id>` (ce qu'il touche). Le code d'un seul symbole : `symbole <id> --code`.
4. **Un besoin, une exigence, une décision** : `document <ID>`, `exigence EF-xxx`, `adr N` ; chercher un thème : `cherche <mots>`.
5. N'ouvrir un fichier source que si la doc ne suffit pas.

## Règles

- Citer ce qu'on avance par son identifiant (`EF-140`, `ADR-0012`, `chemin::Symbole`) : l'utilisateur et le wiki en font des liens.
- Ne rien inventer : un identifiant ou un symbole qui n'est pas dans l'index n'existe pas.
- Répondre en français.
