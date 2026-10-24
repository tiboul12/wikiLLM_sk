---
id: SPEC-24
type: spec
titre: "Paramètres et terminal intégré"
statut: proposé
version: v7
exigences: [EF-147, EF-148]
decisions: [ADR-0013]
---

# SPEC-24 : paramètres et terminal intégré

## Exigences

1. **R1 Paramètres** : section Paramètres (icône en bas de la barre d'activité, `/parametres`) : commande de l'agent (`[ia] agent`, défaut `claude`), consigne de traitement (`[ia] consigne_traitement`, défaut : commencer par un plan et attendre la validation), méthode de lecture du wiki (`[ia] consigne_lecture` : instructions du serveur MCP et préambule des demandes de l'atelier et de la réception), LLM (`[atelier.llm]` : fournisseur, commande, arguments, délai), adresse de l'application (carte de réception). Écrits dans `wikik.toml` en gardant ses commentaires ; une valeur égale au défaut est retirée. Le LLM est remplacé sans relancer le serveur (sauf `--factice`).
2. **R2 Accès** (ADR-0013) : `GET /api/parametres` est libre et dit si le navigateur est propriétaire ; `POST /api/parametres`, `GET /api/terminal` (WebSocket) et `POST /api/terminal/envoyer` exigent la machine locale ou le cookie de la clé (`.wikik/cle`, posé par `/?cle=…`, adresse affichée au lancement), et une origine identique.
3. **R3 Terminal** : un pseudo-terminal par serveur (shell de l'utilisateur, dossier du projet), ouvert à la première connexion, gardé quand le volet se ferme ; les 256 derniers Ko de sa sortie sont rejoués à la reconnexion ; trames binaires pour les octets, trame texte `{cols, rows}` pour la taille. Volet droit à largeur réglable ; plein écran sur téléphone.
4. **R4 Envoi** : `POST /api/terminal/envoyer {texte, mode}` ; `commande` : tapé puis Entrée ; `consigne` : si le shell attend à son invite, `agent '<consigne>'` est lancé, sinon la consigne est collée d'un bloc (collage « bracketed paste ») puis validée. « Traiter avec l'agent ↗ » (cahier de réception, traitement) l'utilise avec la consigne de traitement ; « Lancer l'agent » (volet) avec la commande de l'agent.

## Critères d'acceptation

Test HTTP : paramètres lus et écrits, commentaires de `wikik.toml` gardés, clé posée en cookie, fausse clé ignorée. Contrôle headless : `echo` tapé dans le volet s'affiche ; « Traiter avec l'agent » lance l'agent avec la consigne ; depuis une autre machine sans clé, pas de bouton Terminal, `POST /api/parametres` = 403, WebSocket refusé ; avec la clé, ouvert.
