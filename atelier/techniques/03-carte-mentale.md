---
id: "carte-mentale"
titre: "Carte mentale"
ordre: 3
phase: "ideation"
origine: ["C.A.R.T.E.", "V.E.C.T."]
optionnelle: true
sous_etapes: [{"id": "branches", "genre": "propositions", "cree": "idee", "variantes": {"express": 6, "complet": 12}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "description": {"type": "string"}, "parent": {"type": "string"}}, "required": ["titre", "description", "parent"]}}}, "required": ["elements"]}}, {"id": "liens", "genre": "propositions", "cree": null, "ajoute_a": "carte", "variantes": {"express": 3, "complet": 6}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"de": {"type": "string"}, "vers": {"type": "string"}, "raison": {"type": "string"}}, "required": ["de", "vers", "raison"]}}}, "required": ["elements"]}}, {"id": "carte", "genre": "synthese", "cree": "carte"}]
---

# Carte mentale

## Rôle

Structurer la matière : axes au niveau 1, unités de sens au niveau 2, puis ramifications et liens transversaux proposés par le LLM. La carte est rendue en Mermaid (remplace la génération SVG de V.E.C.T.).

## Consignes

### branches

Lis les axes et unités de sens retenus (outil atelier, types axe et unite). Propose des sous-idées qui découpent finement une unité de sens (niveaux 3 et suivants). « parent » est l'identifiant de l'unité ou de l'axe ramifié.

### liens

Propose des liens logiques entre des branches différentes de la carte (identifiants d'axes, d'unités ou d'idées), avec la raison du lien en une phrase.

## Règles

- Une étape optionnelle : Tibo peut la sauter.
- Les liens relient deux branches distinctes.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "branches": [
    {
      "titre": "Alerte brouillard géolocalisée",
      "description": "Prévenir quand la visibilité chute sur l'itinéraire en cours.",
      "parent": "US-001"
    },
    {
      "titre": "Niveau de difficulté personnalisé",
      "description": "Comparer la difficulté du sentier au niveau déclaré.",
      "parent": "US-002"
    }
  ],
  "liens": [
    {
      "de": "US-001",
      "vers": "US-003",
      "raison": "La peur du brouillard et l'alerte météo répondent au même risque."
    }
  ]
}
```
