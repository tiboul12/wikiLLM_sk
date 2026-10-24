---
id: ADR-0013
type: adr
titre: "Terminal intégré, réservé à la machine locale ou à la clé du lancement"
statut: accepté
date: 2026-10-19
exigences: [EF-147, EF-148]
---

# ADR-0013 : Terminal intégré, réservé à la machine locale ou à la clé du lancement

## Contexte

Tibo veut un volet avec un terminal et l'agent qui développe le projet (REC-022), et que « Traiter les remarques » envoie la consigne dans ce terminal (REC-041). Il ouvre wikik depuis une autre machine du réseau (`[serveur] reseau = true`). Un terminal dans le navigateur est un accès shell complet : servi sans garde sur le réseau, n'importe quelle machine du réseau local l'aurait.

## Décision

- Le serveur ouvre un **pseudo-terminal** (shell de l'utilisateur, dossier du projet) et le relie au navigateur par **WebSocket** ; le front l'affiche avec xterm.js dans un volet droit. Un seul terminal par serveur, gardé ouvert quand le volet se ferme.
- **Accès** : une requête de la machine locale passe ; une requête du réseau doit porter la **clé du serveur**, tirée au hasard au premier lancement, gardée dans `.wikik/cle` (non versionné) et affichée dans l'adresse du message de lancement (`http://10.0.0.21:8484/?cle=…`). La première visite avec la clé pose un cookie `HttpOnly`, `SameSite=Strict`. Même origine exigée comme pour les écritures.
- **Envoi du traitement** : le serveur tape la consigne de traitement (paramètre `[ia] consigne_traitement`) dans le terminal ; il n'exécute rien d'autre.
- Les réglages (LLM, consignes, agent) sont écrits dans `wikik.toml`, en gardant ses commentaires ; leur écriture est réservée au propriétaire, comme le terminal : la commande du LLM est exécutée par le serveur. L'adresse de l'application reste dans la carte de réception (`url_application`, SPEC-23 R7), réglée depuis la même page.

## Conséquences

- Le reste du serveur ne change pas de règle : lecture libre, écritures gardées par l'origine.
- Une clé qui fuit donne un shell : `rm .wikik/cle` et relancer en crée une autre.

## Alternatives écartées

- **Terminal réservé à la machine locale** : inutilisable pour Tibo, qui ouvre wikik depuis une autre machine.
- **Lancer l'agent en tâche de fond (`claude -p`) au lieu d'un terminal** : Tibo veut voir et diriger l'agent, pas recevoir un résultat.
