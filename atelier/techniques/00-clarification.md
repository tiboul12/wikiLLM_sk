---
id: "clarification"
titre: "Clarification"
ordre: 0
phase: "developpement"
origine: ["rappel en développement (ajouté)"]
optionnelle: true
hors_parcours: true
sous_etapes: [{"id": "reponse", "genre": "propositions", "cree": null, "complete": "clarification", "variantes": {"express": 3, "complet": 3}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"piste": {"type": "string"}, "justification": {"type": "string"}, "sources": {"type": "array", "items": {"type": "string"}}}, "required": ["piste", "justification", "sources"]}}}, "required": ["elements"]}}, {"id": "conclusion", "genre": "saisie", "ecran": "clarification"}]
---

# Clarification

## Rôle

Rouvrir la réflexion sur un besoin, une exigence ou un storyboard pendant le développement : l'agent ou Tibo pose une question, le LLM éclaire à partir de l'atelier et du projet, Tibo conclut.

## Consignes

### reponse

Lis la clarification (outil document), son sujet et les sources de ce sujet jusqu'au pitch. Propose des pistes de réponse à la question posée, chacune justifiée par ce que disent l'atelier et le projet ; « sources » cite les identifiants lus. Signale toute contradiction entre l'atelier et le code ou les exigences.

### conclusion

Tibo écrit la conclusion, qui clôt la clarification.

## Règles

- Une clarification a toujours un sujet existant.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "reponse": [
    {
      "piste": "Le seuil de 200 m vient du storyboard SB-001.",
      "justification": "La vignette « Alerte » prévoit 20 minutes d'avance.",
      "sources": [
        "SB-001",
        "EF-001"
      ]
    }
  ]
}
```
