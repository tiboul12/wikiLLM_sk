---
id: SPEC-14
type: spec
titre: "Serveur MCP (v1.1)"
statut: proposé
version: v1.1
exigences: [EF-060, ENF-001]
decisions: [ADR-0002]
---

# SPEC-14 : Serveur MCP

## Problème

En v1, l'agent appelle wikik par le shell. Un serveur MCP lui donne des outils typés, découverts automatiquement, sans dépendre du `PATH` de chaque commande ni du parsing de la sortie shell.

## Exigences

1. **R1** `wikik mcp` parle MCP sur stdio : messages JSON-RPC 2.0, un par ligne ; journaux sur stderr uniquement.
2. **R2** Méthodes : `initialize`, `notifications/initialized`, `ping`, `tools/list`, `tools/call`. Toute autre méthode avec un `id` reçoit l'erreur JSON-RPC `-32601`.
3. **R3** Outils : `carte`, `fichier`, `symbole`, `impact`, `perimes`, `cherche`, `adr`, `exigence`, `ref`, `check`. Les sorties sont **exactement** celles de la CLI en mode texte (même budget de tokens, ENF-001).
4. **R4** L'index est mis à jour avant chaque appel (EF-029). Une erreur d'outil répond `isError: true` avec le message, sans arrêter le serveur.
5. **R5** Version de protocole : le serveur reprend celle demandée par le client.

## Proposition

Implémentation directe (`crates/wikik/src/mcp.rs`, sans dépendance supplémentaire) : le protocole se limite à quelques méthodes, et l'implémenter à la main évite de lier le binaire à l'API d'un SDK encore jeune. Si les besoins grandissent (ressources, notifications), passer au SDK officiel `rmcp`.

Installation dans Claude Code : `claude mcp add wikik -- wikik mcp`, à lancer depuis la racine du projet (le serveur indexe son dossier courant, ou `--racine`).

## Critères d'acceptation

Test d'intégration : lancer `wikik mcp` sur la fixture, envoyer `initialize`, `tools/list`, `tools/call` (`symbole norme`), une méthode inconnue ; vérifier les réponses et l'égalité avec `wikik query symbole norme`.
