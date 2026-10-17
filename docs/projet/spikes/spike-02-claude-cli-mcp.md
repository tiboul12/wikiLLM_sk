---
id: SPIKE-02
type: spike
titre: "claude -p branché sur le MCP de wikik, avec réponse structurée"
statut: terminé
version: v4
date: 2026-10-18
sources: [PLAN-v4, ADR-0011]
---

# SPIKE-02 : `claude -p` + MCP wikik + `--json-schema`

**But** : vérifier, avant les specs, que l'atelier peut appeler le LLM par le CLI local (abonnement), le brancher sur wikik et obtenir des réponses au bon format, assez vite pour une interface.
**Terrain** : Claude Code 2.1.302 sur DeV-CLI, abonnement de Tibo ; serveur MCP `wikik mcp` sur le dépôt wikiK. 5 appels au total.

## Commande retenue

```bash
claude -p "<consigne>" \
  --mcp-config mcp.json --strict-mcp-config \   # seulement le serveur de wikik
  --restricted --tools "" \                      # aucun outil intégré (ni Bash, ni fichiers)
  --allowedTools mcp__wikik \                    # outils wikik autorisés sans confirmation
  --output-format stream-json --verbose \        # événements : init, assistant, rate_limit_event, result
  --json-schema "<schéma de l'artefact>" \       # réponse dans result.structured_output
  --no-session-persistence
```

`--bare` est écarté : il impose une clé d'API (ANTHROPIC_API_KEY) et ne lit pas l'abonnement.

## Résultats

| # | Question | Résultat |
|---|---|---|
| S1 | Le LLM appelle-t-il les outils de wikik sans interaction ? | **Oui.** Il enchaîne `carte`, `symbole`, `fichier` seul (5 à 6 tours) et répond juste (64 fichiers, exigences de l'en-tête de `check.rs`). |
| S2 | `--json-schema` donne-t-il un JSON conforme ? | **Oui sur 5 appels sur 5** : `result.structured_output` respecte le schéma (dont `minItems`/`maxItems`). Le cas d'échec n'a pas été provoqué : prévoir une relance puis un message. |
| S3 | Durée d'un appel | **6 à 10 s** : environ 4 s de démarrage du CLI, puis 2 à 8 s d'API (5 idées : 5,7 s ; avec 5 appels d'outils : 8,5 s). Le flux ne donne pas de progression fine sans `--include-partial-messages` : afficher « en cours » suffit à cette durée. |
| S4 | Limites d'usage | Chaque appel émet un `rate_limit_event` avec l'utilisation des fenêtres de **5 heures** et **7 jours** (46 % et 64 % au moment du test) : l'atelier peut afficher la consommation. Coût indicatif d'un appel : 0,05 à 0,10 USD, inclus dans l'abonnement. |
| S5 | Appels en parallèle | **Oui** : deux appels simultanés aboutissent (6,3 s chacun). |
| S6 | Isolement | Avec `--tools ""`, seuls `StructuredOutput` et les 10 outils `mcp__wikik__*` sont disponibles ; une demande de lecture de `/etc/hostname` est refusée par le LLM lui-même faute d'outil. |

## Ce que ça change au plan

- **ADR-0011 tient** tel quel ; la commande ci-dessus devient le fournisseur par défaut.
- **SPEC-19** : appel asynchrone (6 à 10 s) avec état « en cours » ; lecture du `rate_limit_event` pour afficher la consommation ; relance unique si `structured_output` manque.
- **Coût d'une séance Express** : une douzaine d'appels de génération, plus les relances : de l'ordre de 2 à 3 minutes d'attente cumulée et d'environ 1 USD indicatif.
- **Le serveur MCP devra exposer l'atelier** (phase 4) : le LLM y lira les artefacts des étapes précédentes au lieu de les recevoir dans la consigne.
