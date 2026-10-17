---
id: "scamper"
titre: "SCAMPER"
ordre: 7
phase: "iteration"
origine: ["S.E.C.", "S.C.A.M.P.E.R."]
optionnelle: false
sous_etapes: [{"id": "sujets", "genre": "selection", "parmi": "idee", "nombre": {"express": 1, "complet": 3}}, {"id": "leviers", "genre": "propositions", "pour_chaque": "idee", "cree": "idee", "ecran": "scamper", "variantes": {"express": 7, "complet": 14}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 7, "items": {"type": "object", "properties": {"levier": {"type": "string", "enum": ["S", "C", "A", "M", "P", "E", "R"]}, "titre": {"type": "string"}, "description": {"type": "string"}}, "required": ["levier", "titre", "description"]}}}, "required": ["elements"]}}, {"id": "rapport", "genre": "synthese", "cree": "scamper"}]
---

# SCAMPER

## Rôle

Faire varier les sujets retenus avec les sept leviers : substituer, combiner, adapter, modifier, proposer un autre usage, éliminer, renverser.

## Consignes

### leviers

Pour le sujet indiqué, applique les sept leviers SCAMPER, au moins une idée par levier : S substituer (que remplacer ?), C combiner, A adapter (emprunter à un autre contexte), M modifier (amplifier, réduire), P proposer un autre usage, E éliminer (simplifier), R renverser ou réorganiser.

## Règles

- Au moins une proposition par levier.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "leviers": [
    {
      "levier": "S",
      "titre": "Montre plutôt que téléphone",
      "description": "Alerte sur la montre, mains libres."
    },
    {
      "levier": "C",
      "titre": "Alerte et retour guidé",
      "description": "Combiner l'alerte avec un itinéraire de repli."
    },
    {
      "levier": "A",
      "titre": "Mode phare",
      "description": "Signal lumineux du téléphone pour le groupe."
    },
    {
      "levier": "M",
      "titre": "Alerte graduée",
      "description": "Trois niveaux selon la visibilité."
    },
    {
      "levier": "P",
      "titre": "Pour les refuges",
      "description": "Les refuges diffusent l'alerte aux randonneurs proches."
    },
    {
      "levier": "E",
      "titre": "Sans réseau",
      "description": "Tout fonctionne hors ligne."
    },
    {
      "levier": "R",
      "titre": "Alerte avant le départ",
      "description": "Déconseiller la sortie la veille."
    }
  ]
}
```
