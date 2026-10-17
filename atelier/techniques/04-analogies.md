---
id: "analogies"
titre: "Pensée analogique"
ordre: 4
phase: "ideation"
origine: ["C.I.L.", "P.O.N.T."]
optionnelle: true
sous_etapes: [{"id": "problemes", "genre": "propositions", "cree": "probleme", "variantes": {"express": 4, "complet": 8}, "garder": {"express": 1, "complet": 3}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "description": {"type": "string"}, "sources": {"type": "array", "items": {"type": "string"}}}, "required": ["titre", "description", "sources"]}}}, "required": ["elements"]}}, {"id": "analogies", "genre": "propositions", "pour_chaque": "probleme", "cree": "analogie", "variantes": {"express": 4, "complet": 6}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"domaine": {"type": "string"}, "analogie": {"type": "string"}, "principes": {"type": "array", "items": {"type": "string"}}}, "required": ["domaine", "analogie", "principes"]}}}, "required": ["elements"]}}, {"id": "solutions", "genre": "propositions", "pour_chaque": "probleme", "cree": "idee", "variantes": {"express": 4, "complet": 8}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "description": {"type": "string"}, "principe": {"type": "string"}}, "required": ["titre", "description", "principe"]}}}, "required": ["elements"]}}]
---

# Pensée analogique

## Rôle

Résoudre les problèmes repérés en empruntant à d'autres domaines : choisir les problèmes, ouvrir aux analogies, en extraire les principes, puis les transposer en solutions.

## Consignes

### problemes

Lis les unités de sens et la carte (outil atelier). Consolide une liste de problèmes, défis ou points de friction ; « sources » donne les identifiants d'où chacun vient.

### analogies

Pour le problème indiqué, propose des analogies prises dans des domaines éloignés (nature, autres industries, systèmes humains). Pour chacune, décris le mécanisme et extrais deux à quatre principes transférables.

### solutions

Rappelle le problème et les principes des analogies retenues. Applique chaque principe au problème pour proposer des solutions concrètes ; « principe » cite celui qui l'inspire.

## Règles

- Une étape optionnelle.
- Chaque solution cite son principe d'origine.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "problemes": [
    {
      "titre": "Se perdre par mauvaise visibilité",
      "description": "Le randonneur perd le sentier quand le brouillard tombe.",
      "sources": [
        "US-001"
      ]
    }
  ],
  "analogies": [
    {
      "domaine": "Navigation maritime",
      "analogie": "Les phares et les cornes de brume",
      "principes": [
        "Signal redondant visuel et sonore",
        "Repères fixes connus à l'avance"
      ]
    }
  ],
  "solutions": [
    {
      "titre": "Balises sonores virtuelles",
      "description": "Le téléphone émet un son quand on s'écarte du tracé.",
      "principe": "Signal redondant visuel et sonore"
    }
  ]
}
```
