---
id: ADR-0014
type: adr
titre: "Le serveur du projet tourne en arrière-plan, jusqu'à son bouton d'arrêt"
statut: accepté
date: 2026-10-20
exigences: [EF-130]
---

# ADR-0014 : le serveur du projet tourne en arrière-plan, jusqu'à son bouton d'arrêt

## Contexte

Tibo lance `wikik` sur sa machine de développement par SSH. Quand la connexion se ferme, le terminal envoie SIGHUP au serveur, qui s'arrête. Il veut que le serveur, une fois lancé, reste allumé jusqu'à l'arrêt de la machine ou jusqu'à ce qu'il clique sur un bouton d'arrêt, sans commande de plus (ADR-0012).

## Décision

- `wikik` démarre le serveur **détaché** : un processus dans une nouvelle session (`setsid`), sans terminal de contrôle, sa sortie dans `.wikik/serveur.log`. `wikik` attend qu'il réponde, affiche ses adresses et rend la main.
- Relancé alors que le serveur du projet tourne, `wikik` réaffiche ses adresses sans en démarrer un second.
- Arrêt : bouton ⏻ « Arrêter le serveur » en bas de la barre d'activité, réservé au propriétaire (ADR-0013), ou SIGTERM. Le port noté dans `.wikik/serveur.json` est retiré à l'arrêt.
- `WIKIK_PREMIER_PLAN=1` garde le serveur au premier plan : c'est ainsi que `wikik` lance le serveur détaché, et que les tests le pilotent. Ce n'est pas une option à retenir pour l'utilisateur.

## Conséquences

- Fermer la connexion SSH, ou le terminal, ne coupe plus le serveur ; un redémarrage de la machine, si (pas de service système).
- Les terminaux intégrés (ADR-0013) vivent aussi longtemps que le serveur.

## Alternatives écartées

- **Service systemd** : survit au redémarrage, mais demande une installation et une commande de plus par projet (« usine à gaz »).
- **`nohup wikik &` dans la doc** : à retenir et à refaire à chaque fois ; un oubli coupe le serveur.
