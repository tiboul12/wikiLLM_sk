---
id: ADR-0016
type: adr
titre: "Édition des documents dans le navigateur, avec un journal d'annulation"
statut: proposé
date: 2026-10-22
exigences: [EF-158, EF-159]
---

# ADR-0016 : édition des documents dans le navigateur, avec un journal d'annulation

## Contexte

En v1, l'édition dans le navigateur avait été écartée (Q5) : les documents se modifient dans l'éditeur, l'agent les écrit. Tibo veut maintenant valider et corriger un document depuis la relecture (REC-053, REC-056) et annuler par Ctrl+Z (REC-065).

## Décision

- Le serveur écrit les documents de `docs/` sur demande du propriétaire (ADR-0013) : remplacement complet du fichier (frontmatter et corps), ou changement d'un champ du frontmatter (statut). Le chemin doit rester sous `docs/` et finir en `.md`.
- Chaque écriture faite par le serveur pour l'interface (documents, atelier, réception, paramètres) passe par un **journal** en mémoire : avant d'écrire, il garde le contenu précédent de chaque fichier touché (ou son absence). `POST /api/annuler` restaure l'état d'avant la dernière action, `POST /api/retablir` la rejoue. Le journal garde les 100 dernières actions et se vide au redémarrage.
- Le front envoie Ctrl+Z / Ctrl+Y au serveur quand le focus n'est pas dans un champ de saisie (où le navigateur garde son annulation native), puis recharge l'état.
- Une annulation qui trouverait un fichier modifié depuis par quelqu'un d'autre (contenu différent de celui écrit) est refusée, avec le nom du fichier.

## Conséquences

- Pas de perte silencieuse : git reste le filet de sécurité, le journal sert au geste immédiat.
- Les écritures de l'agent (hors serveur) ne sont pas dans le journal.

## Alternatives écartées

- **Annuler par git (`git checkout`)** : annule aussi les modifications non commitées de l'agent.
- **Annulation propre à chaque écran** : chaque écran réinventerait la sienne ; Ctrl+Z n'aurait pas le même effet partout.
