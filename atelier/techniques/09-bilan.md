---
id: "bilan"
titre: "Bilan des besoins"
ordre: 9
phase: "bilan"
origine: ["B.I.L.A.N."]
optionnelle: false
sous_etapes: [{"id": "besoins", "genre": "propositions", "cree": "besoin", "variantes": {"express": 10, "complet": 20}, "schema": {"type": "object", "properties": {"elements": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {"titre": {"type": "string"}, "description": {"type": "string"}, "qui": {"type": "string"}, "pourquoi": {"type": "string"}, "categorie": {"type": "string"}, "priorite": {"type": "string", "enum": ["must", "should", "could", "wont"]}, "sources": {"type": "array", "items": {"type": "string"}}}, "required": ["titre", "description", "categorie", "priorite", "sources"]}}}, "required": ["elements"]}}, {"id": "tri", "genre": "saisie", "ecran": "tri-besoins"}, {"id": "bdr", "genre": "synthese", "cree": "bdr"}]
---

# Bilan des besoins

## Rôle

Consolider toute la cascade en besoins dédoublonnés, catégorisés et priorisés (MoSCoW, selon la valeur et non la seule récurrence), puis produire la BDR 1 et la BDR 2 du projet.

## Consignes

### besoins

Parcours tous les artefacts retenus de l'atelier (outil atelier, puis document au besoin) : unités de sens, idées, analyses, SCAMPER, storyboards et leurs artefacts. Extrais les besoins actionnables, fusionne les doublons, propose une catégorie (fonctionnel, technique, UX, piste d'analyse…) et une priorité MoSCoW selon la valeur pour la cible du pitch. « qui » nomme la personne ou le rôle concerné ; « pourquoi » donne l'irritant ou l'enjeu, avec la preuve (verbatim, storyboard) quand elle existe. La description dit ce qu'il faut pouvoir faire ou savoir, sans solution technique. « sources » cite TOUS les identifiants d'origine (principe de traçabilité).

### tri

Réordonner, recatégoriser et reprioriser les besoins retenus.

## Règles

- Chaque besoin cite au moins une source.
- Exhaustivité : chaque storyboard retenu alimente au moins un besoin.

## Exemple

Projet « MyRandonnée » (exemple du framework d'origine) : réponses attendues par sous-étape, utilisées par le fournisseur factice et les tests.

```json
{
  "besoins": [
    {
      "titre": "Être alerté d'une perte de visibilité",
      "description": "Le randonneur est prévenu avant que le brouillard ne l'atteigne.",
      "qui": "Randonneur occasionnel, seul en montagne.",
      "pourquoi": "« j'ai peur de me perdre quand le brouillard tombe » (US-001).",
      "categorie": "fonctionnel",
      "priorite": "must",
      "sources": [
        "US-003",
        "IDEE-001",
        "SB-001"
      ]
    },
    {
      "titre": "Fonctionner hors réseau",
      "description": "Les alertes reposent sur des données téléchargées avant le départ.",
      "categorie": "technique",
      "priorite": "must",
      "sources": [
        "IDEE-006"
      ]
    }
  ]
}
```
