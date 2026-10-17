---
id: "free-writing"
titre: "Free writing"
ordre: 2
phase: "ideation"
origine: ["C.I.S.", "P.A.R.S."]
optionnelle: false
sous_etapes: [{"id": "axes", "genre": "propositions", "cree": "axe", "variantes": {"express": 4, "complet": 6}, "garder": {"express": 1, "complet": 4}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "description": {"type": "string"}}, "required": ["titre", "description"]}}}, "required": ["elements"]}}, {"id": "ecriture", "genre": "saisie", "pour_chaque": "axe", "cree": "ecrit", "ecran": "free-writing", "duree_minutes": {"express": 10, "complet": 15}}, {"id": "recolte", "genre": "propositions", "pour_chaque": "ecrit", "cree": "unite", "variantes": {"express": 12, "complet": 20}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"verbatim": {"type": "string"}, "qualification": {"type": "string", "enum": ["Idée", "Problème", "Besoin", "Émotion", "Frustration", "Contrainte", "Question"]}}, "required": ["verbatim", "qualification"]}}}, "required": ["elements"]}}]
---

# Free writing

## Rôle

Faire émerger la matière brute : Tibo écrit sans s'arrêter sur un axe d'analyse, puis le LLM récolte et qualifie les unités de sens, mot pour mot.

## Consignes

### axes

Lis le pitch (outil document PITCH). Propose des axes d'analyse standards pertinents pour ce projet (usages, irritants, contexte, économie, technique, émotions…). Pour chacun, un titre court et une phrase qui dit ce que l'écriture libre doit explorer.

### ecriture

Écrire sans s'arrêter, sans jugement ni censure, pendant la durée choisie, sur l'axe affiché.

### recolte

Lis le texte libre (outil document sur l'identifiant indiqué). Extrais chaque unité de sens MOT POUR MOT, sans reformuler, résumer ni corriger, et qualifie-la : Idée, Problème, Besoin, Émotion, Frustration, Contrainte ou Question.

## Règles

- Principe de non-altération : un verbatim doit apparaître tel quel dans le texte libre ; sinon la proposition est refusée.
- Une session d'écriture par axe retenu.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "axes": [
    {
      "titre": "Sécurité en montagne",
      "description": "Ce qui fait peur ou met en danger lors d'une sortie."
    },
    {
      "titre": "Préparation",
      "description": "Comment on choisit et prépare une randonnée aujourd'hui."
    },
    {
      "titre": "Partage",
      "description": "Raconter, montrer, inviter d'autres randonneurs."
    },
    {
      "titre": "Équipement",
      "description": "Ce qu'on emporte et ce qu'on oublie."
    }
  ],
  "recolte": [
    {
      "verbatim": "j'ai peur de me perdre quand le brouillard tombe",
      "qualification": "Émotion"
    },
    {
      "verbatim": "aucune appli ne me dit si le sentier est trop dur pour moi",
      "qualification": "Problème"
    },
    {
      "verbatim": "une alerte si la météo tourne",
      "qualification": "Idée"
    }
  ]
}
```
