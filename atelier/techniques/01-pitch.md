---
id: "pitch"
titre: "Pitch"
ordre: 1
phase: "cadrage"
origine: ["étape 0 (ajoutée)"]
optionnelle: false
sous_etapes: [{"id": "pitch", "genre": "saisie", "cree": "pitch", "ecran": "pitch", "brouillon": true, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "maxItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "probleme": {"type": "string"}, "cible": {"type": "string"}, "valeur": {"type": "string"}, "contraintes": {"type": "string"}, "criteres_succes": {"type": "string"}, "hors_sujet": {"type": "string"}}, "required": ["titre", "probleme", "cible", "valeur", "contraintes", "criteres_succes", "hors_sujet"]}}}, "required": ["elements"]}}]
---

# Pitch

## Rôle

Poser le projet en six réponses courtes : le problème, pour qui, la valeur apportée, les contraintes, les critères de succès et ce qui n'est pas le sujet. Le pitch alimente la BDR 1 et sert de contexte à toutes les étapes suivantes.

## Consignes

### pitch

À partir de la description libre que Tibo vient d'écrire (champ « description » du pitch, lisible avec l'outil document PITCH s'il existe déjà), propose un brouillon de pitch : un titre de projet court, puis une à trois phrases pour chaque champ. N'invente aucun fait absent de la description : quand une information manque, écris « à préciser ».

## Règles

- Une réponse par champ, sans solution technique dans « problème ».
- « hors_sujet » liste au moins un élément.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "pitch": [
    {
      "titre": "MyRandonnée",
      "probleme": "Les randonneurs amateurs préparent mal leurs sorties : itinéraires inadaptés à leur niveau, météo ignorée.",
      "cible": "Randonneurs occasionnels, seuls ou en petit groupe.",
      "valeur": "Préparer une sortie adaptée en quelques minutes et la partager.",
      "contraintes": "Usage hors ligne en montagne ; données cartographiques libres.",
      "criteres_succes": "Une sortie préparée en moins de 5 minutes ; 3 randonneurs testeurs satisfaits.",
      "hors_sujet": "Alpinisme et via ferrata."
    }
  ]
}
```
