---
id: "cubing"
titre: "Cubing"
ordre: 6
phase: "raffinage"
origine: ["S.A.S.", "C.A.R."]
optionnelle: false
sous_etapes: [{"id": "idees", "genre": "selection", "parmi": "idee", "nombre": {"express": 3, "complet": 6}, "preselection": "5w1h"}, {"id": "faces", "genre": "propositions", "pour_chaque": "idee", "cree": "analyse", "methode": "cubing", "variantes": {"express": 1, "complet": 1}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "maxItems": 1, "items": {"type": "object", "properties": {"decrire": {"type": "string"}, "comparer": {"type": "string"}, "associer": {"type": "string"}, "analyser": {"type": "string"}, "appliquer": {"type": "string"}, "argumenter": {"type": "string"}}, "required": ["decrire", "comparer", "associer", "analyser", "appliquer", "argumenter"]}}}, "required": ["elements"]}}]
---

# Cubing

## Rôle

Regarder chaque idée sous six faces : décrire, comparer, associer, analyser, appliquer, argumenter.

## Consignes

### faces

Lis l'idée indiquée, son analyse 5W1H si elle existe et le contexte de l'atelier. Formule une proposition pour chacune des six faces du Cubing (décrire, comparer, associer, analyser, appliquer, argumenter), deux à quatre phrases chacune.

## Règles

- Par défaut, les idées sont celles analysées en 5W1H.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "faces": [
    {
      "decrire": "Alerte locale de perte de visibilité.",
      "comparer": "Plus ciblée qu'une alerte météo générale.",
      "associer": "Navigation, sécurité, prévision.",
      "analyser": "Dépend de la qualité de la prévision hors ligne.",
      "appliquer": "Randonnée, raquettes, trail.",
      "argumenter": "Réduit le risque principal cité par les testeurs."
    }
  ]
}
```
