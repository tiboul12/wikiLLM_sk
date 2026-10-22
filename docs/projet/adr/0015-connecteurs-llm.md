---
id: ADR-0015
type: adr
titre: "Connecteurs LLM : CLI d'abonnement par préréglages, modèles locaux par HTTP"
statut: proposé
date: 2026-10-22
exigences: [EF-155]
---

# ADR-0015 : connecteurs LLM : CLI d'abonnement par préréglages, modèles locaux par HTTP

## Contexte

Jusqu'en v7, le LLM de l'atelier et de la réception est une commande (`claude -p …`) dont Tibo règle les arguments à la main (ADR-0011). Il veut choisir entre Claude, Antigravity, Mistral et les modèles locaux de Shelley, puis le modèle et l'effort (REC-057.2), « comme sur ATS-cv-editor » (ATS ADR-0009 : CLI local déjà connecté, préréglages, test de connexion, liste des modèles).

## Décision

- `[atelier.llm] connecteur` choisit un connecteur : `claude` (Claude Code), `agy` (Antigravity), `vibe` (Mistral Vibe), `ollama`, `llama-cpp`, `cli` (commande libre, arguments à la main, le mode d'avant), `factice`.
- Les connecteurs CLI construisent leurs arguments eux-mêmes à partir de `modele` et `effort` : aucun argument à saisir. Seul `claude` reçoit le serveur MCP de wikik et un schéma JSON (`--json-schema`) ; `agy` reçoit aussi le schéma (`--json-schema`), `vibe` le reçoit dans la consigne ; leur réponse JSON est extraite du texte.
- Les connecteurs locaux (`ollama`, `llama-cpp`) parlent HTTP à l'adresse `[atelier.llm] url` (par défaut `http://localhost:11434` et `http://127.0.0.1:8090`, comme Shelley) : `/api/chat` avec `format` = schéma pour Ollama, `/v1/chat/completions` pour llama.cpp. Un modèle local n'a pas d'outils MCP : la consigne lui donne la carte du projet et les documents cités.
- « Tester » lance un appel minimal ; « Modèles » liste les modèles : alias documentés pour `claude`, `agy models`, `/api/tags` d'Ollama, `/v1/models` de llama.cpp.
- Aucune clé d'API n'est gérée par wikik : l'authentification reste celle du CLI ou du serveur local.

## Conséquences

- Les anciennes configurations (`fournisseur = "cli"` avec arguments) continuent de marcher via le connecteur `cli` ; le fichier n'est réécrit qu'à la première modification à l'écran.
- La qualité des propositions dépend du connecteur : sans MCP, le LLM ne lit que ce qu'on lui donne.

## Alternatives écartées

- **Clés d'API par fournisseur** : facturation à l'usage et secrets à stocker ; Tibo travaille par abonnements et par sa machine locale.
- **Passer par la passerelle de Shelley Administration** : elle n'expose pas d'API de conversation ; wikik lit directement les moteurs qu'elle configure.
