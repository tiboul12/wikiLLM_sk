---
id: "storyboard"
titre: "Storyboard"
ordre: 8
phase: "scenarisation"
origine: ["S.E.L.", "R.E.C.I.T."]
optionnelle: false
sous_etapes: [{"id": "scenarios", "genre": "propositions", "cree": "storyboard", "variantes": {"express": 3, "complet": 6}, "garder": {"express": 2, "complet": 5}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "scenario": {"type": "string"}, "sources": {"type": "array", "items": {"type": "string"}}}, "required": ["titre", "scenario", "sources"]}}}, "required": ["elements"]}}, {"id": "cadre", "genre": "propositions", "pour_chaque": "storyboard", "cree": null, "complete": "storyboard", "variantes": {"express": 1, "complet": 1}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "maxItems": 1, "items": {"type": "object", "properties": {"personnage": {"type": "string"}, "scene": {"type": "string"}, "intrigue": {"type": "string"}}, "required": ["personnage", "scene", "intrigue"]}}}, "required": ["elements"]}}, {"id": "vignettes", "genre": "propositions", "pour_chaque": "storyboard", "cree": null, "complete": "storyboard", "ecran": "storyboard", "variantes": {"express": 5, "complet": 8}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "description": {"type": "string"}, "prompt": {"type": "string"}}, "required": ["titre", "description", "prompt"]}}}, "required": ["elements"]}}, {"id": "artefacts", "genre": "propositions", "pour_chaque": "storyboard", "cree": null, "complete": "storyboard", "variantes": {"express": 4, "complet": 8}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"nom": {"type": "string"}, "genre": {"type": "string", "enum": ["page", "fenetre", "rapport", "notification", "donnee", "service", "autre"]}, "vignette": {"type": "string"}, "description": {"type": "string"}}, "required": ["nom", "genre", "vignette", "description"]}}}, "required": ["elements"]}}, {"id": "recit", "genre": "synthese", "cree": "storyboard"}]
---

# Storyboard

## Rôle

Raconter des scénarios d'usage en vignettes, puis en tirer les artefacts à produire (pages, fenêtres, rapports) : c'est le pont vers les exigences. Les vignettes sont décrites en texte avec leur prompt (la génération d'images de R.E.C.I.T. est reportée).

## Consignes

### scenarios

Lis les idées retenues, les analyses et le SCAMPER (outil atelier). Propose des scénarios fonctionnels : une séquence d'actions d'un utilisateur, en 3 à 6 phrases ; « sources » cite les idées et analyses d'origine.

### cadre

Pour le scénario indiqué, propose le personnage principal, la scène (contexte, lieu) et l'intrigue (le problème à résoudre ou l'objectif).

### vignettes

Découpe le scénario en vignettes successives : pour chacune, un titre, ce qui se passe à l'écran ou dans la situation, et un prompt d'illustration structuré (sujet, cadre, style).

### artefacts

À partir des vignettes du storyboard, liste les artefacts à produire (pages, fenêtres, rapports, notifications, données, services). « vignette » cite le titre de la vignette d'où vient l'artefact.

## Règles

- Chaque artefact cite sa vignette source.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "scenarios": [
    {
      "titre": "Alerte brouillard en sortie",
      "scenario": "Léa randonne seule. Le brouillard arrive. Son téléphone vibre et propose un retour guidé. Elle rejoint le parking.",
      "sources": [
        "IDEE-001"
      ]
    }
  ],
  "cadre": [
    {
      "personnage": "Léa, 34 ans, randonneuse occasionnelle",
      "scene": "Sentier de crête, fin d'après-midi",
      "intrigue": "Rentrer avant que la visibilité ne disparaisse"
    }
  ],
  "vignettes": [
    {
      "titre": "Départ",
      "description": "Léa télécharge l'itinéraire et la prévision.",
      "prompt": "randonneuse au parking, téléphone en main, style croquis"
    },
    {
      "titre": "Alerte",
      "description": "Le téléphone vibre : visibilité en baisse dans 20 minutes.",
      "prompt": "écran d'alerte sur fond de crête brumeuse"
    }
  ],
  "artefacts": [
    {
      "nom": "Écran d'alerte visibilité",
      "genre": "notification",
      "vignette": "Alerte",
      "description": "Alerte plein écran avec temps restant."
    }
  ]
}
```
