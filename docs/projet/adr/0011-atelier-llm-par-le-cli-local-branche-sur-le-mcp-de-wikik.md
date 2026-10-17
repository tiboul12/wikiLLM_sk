---
id: ADR-0011
type: adr
titre: "Atelier : LLM par le CLI local, branché sur le MCP de wikik"
statut: accepté
date: 2026-10-17
remplace: []
exigences: []
---

# ADR-0011 : Atelier : LLM par le CLI local, branché sur le MCP de wikik

## Contexte

L'atelier fait proposer des variantes par un LLM à chaque étape. Tibo dispose d'un abonnement Claude ; ATS appelle déjà l'IA par le CLI local (son ADR-0009). Le LLM doit pouvoir lire l'atelier et le projet (décision B4).

## Décision

- L'appel passe par une **couche fournisseur générique** : par défaut le CLI `claude` en mode non interactif (`claude -p`), configurable dans `wikik.toml` (commande, modèle, arguments). Aucune clé d'API n'est stockée.
- Chaque appel branche le **serveur MCP de wikik** (`--mcp-config`) et **n'autorise que ses outils** (`--allowedTools`), en lecture : le LLM consulte, seul l'atelier écrit.
- La réponse attendue est décrite par un **schéma JSON** (`--json-schema`) propre à la technique ; l'atelier la valide avant de l'afficher.
- Un **fournisseur factice** remplace le CLI dans les tests.

## Conséquences

- Coût inclus dans l'abonnement, sous ses limites d'usage (à mesurer, spike S4).
- Le contexte envoyé reste petit : la fiche de la technique et les identifiants utiles ; le LLM va chercher le reste par MCP.

## Alternatives écartées

- **API Anthropic avec clé** : plus rapide, facturée à l'usage, clé à protéger ; reste possible par la couche fournisseur.
- **Tout le contexte dans le prompt** : saturation, comme dans le chat (diagnostic du 30/09).
