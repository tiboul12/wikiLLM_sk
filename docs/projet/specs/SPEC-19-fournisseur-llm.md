---
id: SPEC-19
type: spec
titre: "Atelier : fournisseur LLM"
statut: proposé
version: v4
exigences: [EF-104, EF-111, ENF-007, ENF-008]
decisions: [ADR-0011]
---

# SPEC-19 : fournisseur LLM

## Exigences

1. **R1** Trait `Fournisseur` : `proposer(consigne, schema) -> Reponse { donnees: JSON, consommation: Option<Consommation>, duree_ms }`.
2. **R2** Fournisseur `cli` (défaut) : commande et arguments dans `wikik.toml`, avec les jetons `{consigne}`, `{schema}`, `{mcp}` remplacés à l'appel ; il lit le flux `stream-json`, prend `result.structured_output`, et `rate_limit_event` pour la consommation.
3. **R3** Fournisseur `factice` : renvoie l'exemple de la fiche ; utilisé par les tests et par `wikik atelier --factice`.
4. **R4** Délai maximum 180 s ; une relance si `structured_output` manque ou ne respecte pas le schéma ; ensuite, erreur lisible à l'écran.
5. **R5** Isolement (ENF-008) : la commande par défaut désactive tous les outils intégrés et n'autorise que les outils de wikik ; le fichier `--mcp-config` est écrit par l'atelier dans `.wikik/atelier/mcp.json` et pointe sur le binaire courant.
6. **R6** Journal : `.wikik/atelier/journal.jsonl` (date, étape, durée, succès, consommation), non versionné.

## Configuration par défaut

```toml
[atelier.llm]
fournisseur = "cli"
commande = "claude"
arguments = ["-p", "{consigne}", "--mcp-config", "{mcp}", "--strict-mcp-config", "--restricted",
             "--tools", "", "--allowedTools", "mcp__wikik", "--output-format", "stream-json",
             "--verbose", "--json-schema", "{schema}", "--no-session-persistence"]
```

## Critères d'acceptation

1. Test : un faux exécutable qui écrit un flux `stream-json` est analysé (données, consommation, durée).
2. Test : une réponse sans `structured_output` déclenche exactement une relance.
