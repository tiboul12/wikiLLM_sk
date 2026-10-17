---
id: "roadmap"
titre: "Roadmap"
ordre: 11
phase: "planification"
origine: ["R.O.A.D."]
optionnelle: false
sous_etapes: [{"id": "horizons", "genre": "propositions", "cree": null, "ecran": "roadmap", "variantes": {"express": 1, "complet": 1}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "maxItems": 1, "items": {"type": "object", "properties": {"maintenant": {"type": "object", "properties": {"objectif": {"type": "string"}, "recit": {"type": "string"}, "besoins": {"type": "array", "items": {"type": "string"}}}, "required": ["objectif", "recit", "besoins"]}, "ensuite": {"type": "object", "properties": {"objectif": {"type": "string"}, "recit": {"type": "string"}, "besoins": {"type": "array", "items": {"type": "string"}}}, "required": ["objectif", "recit", "besoins"]}, "plus_tard": {"type": "object", "properties": {"objectif": {"type": "string"}, "recit": {"type": "string"}, "besoins": {"type": "array", "items": {"type": "string"}}}, "required": ["objectif", "recit", "besoins"]}}, "required": ["maintenant", "ensuite", "plus_tard"]}}}, "required": ["elements"]}}, {"id": "roadmap", "genre": "synthese", "cree": "roadmap"}]
---

# Roadmap

## Rôle

Répartir les besoins en trois horizons (Maintenant, Ensuite, Plus tard) avec un objectif stratégique et un récit pour chacun.

## Consignes

### horizons

Lis les besoins du projet et leurs priorités. Propose une répartition par défaut (must → Maintenant, should → Ensuite, could → Plus tard), puis, pour chaque horizon, un objectif stratégique et un récit de deux à trois phrases qui justifie ce découpage. N'ajoute aucun besoin nouveau (principe de filiation).

## Règles

- Uniquement des besoins existants.
- Chaque besoin retenu figure dans un seul horizon.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "horizons": [
    {
      "maintenant": {
        "objectif": "Sécuriser la sortie",
        "recit": "La première version répond à la peur principale : se perdre.",
        "besoins": [
          "B-001",
          "B-002"
        ]
      },
      "ensuite": {
        "objectif": "Partager",
        "recit": "Une fois la sécurité acquise, on ouvre au partage.",
        "besoins": []
      },
      "plus_tard": {
        "objectif": "Communauté",
        "recit": "Les refuges et les groupes.",
        "besoins": []
      }
    }
  ]
}
```
