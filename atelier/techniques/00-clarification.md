---
id: "clarification"
titre: "Clarification"
ordre: 0
phase: "developpement"
origine: ["rappel en développement (ajouté)"]
optionnelle: true
hors_parcours: true
sous_etapes: [{"id": "reponse", "genre": "propositions", "cree": null, "complete": "clarification", "variantes": {"express": 3, "complet": 3}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"piste": {"type": "string"}, "justification": {"type": "string"}, "sources": {"type": "array", "items": {"type": "string"}}, "hors_sujet": {"type": "string"}}, "required": ["piste", "justification", "sources"]}}}, "required": ["elements"]}}]
---

# Clarification

## Rôle

Rouvrir la réflexion sur un besoin, une exigence ou un storyboard pendant le développement : l'agent pose une question (ou Tibo lance la clarification d'un document, sans rien écrire), le LLM propose des pistes à partir de l'atelier et du projet, Tibo garde, corrige ou rejette ; les pistes gardées font la réponse, et Tibo déclare la clarification finie.

## Consignes

### reponse

Lis la clarification (outil document), son sujet et les sources de ce sujet jusqu'au pitch. Juge d'abord si la question porte sur le sujet : si elle est hors sujet (elle parle de l'outil, d'un autre document, ou n'a pas de sens pour ce sujet), dis-le en une phrase dans « hors_sujet » de la première piste, et ramène au sujet. Propose ensuite des pistes de réponse, chacune justifiée par ce que disent l'atelier et le projet ; « sources » cite les identifiants lus, les chemins de fichier et les routes des écrans (`/clarifications`). Signale toute contradiction entre l'atelier et le code ou les exigences.

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
