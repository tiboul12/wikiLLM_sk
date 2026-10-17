---
id: "exigences"
titre: "Exigences"
ordre: 10
phase: "specification"
origine: ["étape ajoutée (cahier des charges)"]
optionnelle: false
sous_etapes: [{"id": "besoins", "genre": "selection", "parmi": "besoin", "nombre": {"express": 50, "complet": 200}, "preselection": "tous"}, {"id": "exigences", "genre": "propositions", "pour_chaque": "besoin", "cree": "exigence", "ecran": "exigences", "variantes": {"express": 2, "complet": 4}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "enonce": {"type": "string"}, "critere": {"type": "string"}, "priorite": {"type": "string", "enum": ["must", "should", "could", "wont"]}, "sources": {"type": "array", "items": {"type": "string"}}}, "required": ["titre", "enonce", "critere", "priorite", "sources"]}}}, "required": ["elements"]}}, {"id": "cahier", "genre": "synthese", "cree": "cahier-des-charges"}]
---

# Exigences

## Rôle

Décliner chaque besoin en exigences testables (énoncé, critère d'acceptation, priorité), reliées aux storyboards et analyses qui les justifient : c'est le cahier des charges du projet.

## Consignes

### exigences

Pour le besoin indiqué (outil document) et ses sources, rédige des exigences fonctionnelles : un énoncé « Le système doit … », un critère d'acceptation observable et testable, une priorité. « sources » cite les storyboards, artefacts et analyses qui la justifient. Aucune solution technique (pas de choix de langage, de base, de framework).

## Règles

- Chaque exigence cite son besoin.
- Un critère d'acceptation est vérifiable par un test ou une observation.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "exigences": [
    {
      "titre": "Alerte de visibilité",
      "enonce": "Le système doit alerter l'utilisateur lorsque la visibilité prévue sur son itinéraire passe sous 200 m dans les 30 minutes.",
      "critere": "Avec une prévision simulée à 150 m dans 20 minutes, une alerte s'affiche et vibre.",
      "priorite": "must",
      "sources": [
        "SB-001"
      ]
    }
  ]
}
```
