---
id: PLAN-v6
type: plan
titre: "Plan v6 : un seul serveur, accueil et sections"
statut: validé
version: v6
sources: [BDR-v6-2, CDC-v6, ADR-0012]
---

# Plan v6 : un seul serveur, accueil et sections

**En une phrase** : `wikik` lance le serveur du projet, qui réunit Produit, Projet, Atelier et Réception derrière une page d'accueil et une barre d'activité à gauche.

## Suivi

| Phase | État | Date |
|---|---|---|
| 0 BDR, CDC, ADR-0012 | ✅ | 2026-10-19 |
| 1 Serveur unique, commande unique | ✅ `serve`, `atelier`, `reception` et `--autoriser-reseau` supprimés ; un fournisseur LLM partagé ; carte de réception enregistrée au premier geste (le lancement n'écrit rien) ; `[atelier] url` et `url_wiki` supprimés, adresse de rappel = port réellement pris | 2026-10-19 |
| 2 Accueil et sections | ✅ accueil à tuiles, barre d'activité (badges : remarques et clarifications ouvertes ; barre du bas sur téléphone), panneau par section, accueils Produit et Projet ; site exporté : Accueil, Produit, Projet ; headless clair / sombre / téléphone sans erreur ni débordement | 2026-10-19 |
