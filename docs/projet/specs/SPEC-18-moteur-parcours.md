---
id: SPEC-18
type: spec
titre: "Atelier : fiches de technique et moteur de parcours"
statut: proposé
version: v4
exigences: [EF-101, EF-102, EF-103]
decisions: [ADR-0009]
---

# SPEC-18 : fiches et moteur de parcours

## Problème

Remplacer le méga-prompt par des fiches déclaratives que le moteur enchaîne, avec le principe « le LLM propose en lot, Tibo choisit » (B6), tout en respectant les moments où c'est Tibo qui produit (texte libre, pitch).

## Fiche de technique

Un fichier par technique dans `atelier/techniques/`, embarqué dans le binaire : frontmatter YAML (machine) et corps Markdown (consignes lisibles).

```yaml
id: scamper
titre: SCAMPER
ordre: 7
phase: iteration            # ideation | raffinage | iteration | scenarisation | bilan | specification | planification
origine: [S.E.C., S.C.A.M.P.E.R.]
optionnelle: false          # une étape optionnelle peut être sautée (carte mentale, analogies)
sous_etapes:
  - id: sujets
    genre: selection         # saisie | propositions | selection | synthese
    parmi: idee              # type d'artefact proposé au choix
    nombre: { express: 1, complet: 3 }
  - id: leviers
    genre: propositions
    pour_chaque: sujet       # répété pour chaque sujet choisi
    cree: idee               # type créé pour chaque proposition gardée
    variantes: { express: 7, complet: 14 }
    schema: { … }            # schéma JSON de la réponse (une liste d'objets)
  - id: rapport
    genre: synthese
    cree: scamper
```

Le corps de la fiche porte, sous des titres fixes : `## Rôle` (pour Tibo), `## Consignes` par sous-étape (`### leviers`, envoyées au LLM), `## Règles` (validation), `## Exemple` (MyRandonnée, en YAML : sert au fournisseur factice et aux tests).

## Genres de sous-étapes

| Genre | Déroulé |
|---|---|
| `saisie` | Formulaire ou éditeur ; Tibo écrit ; l'atelier enregistre (statut `retenu`). Le LLM peut, sur demande, proposer un brouillon que Tibo corrige. |
| `propositions` | Un appel au LLM (SPEC-19) rend `variantes` éléments au schéma ; chacun s'affiche en carte : **garder**, **corriger** (puis garder), **rejeter**, et **redemander** pour le lot (nouvel appel qui reçoit les titres déjà vus pour éviter les doublons). |
| `selection` | Liste des artefacts du type `parmi` ; Tibo coche (bornes `nombre`). Aucun appel. |
| `synthese` | Assemblage sans LLM des artefacts de l'étape en un artefact récapitulatif (ou, pour le bilan et les exigences, en fichiers de `docs/projet/`). |

## Parcours

- Ordre : Pitch, Free writing, Carte mentale, Analogies, 5W1H, Cubing, SCAMPER, Storyboard, Bilan, Exigences, Roadmap.
- État dans `docs/atelier/parcours.md` (versionné) : mode, état de chaque étape (`a_faire`, `en_cours`, `faite`, `sautee`), sous-étape et boucle courantes. Une étape peut être rouverte à tout moment ; le mode (Express ou Complet) se change par étape.
- Plusieurs parcours (EF-145) : `PARCOURS` (ci-dessus) et `PARCOURS-2`, `PARCOURS-3`… dans `docs/atelier/parcours-n/` (`parcours.md`, `pitch.md` = `PITCH-n`, `carte.md` = `CARTE-n`), chacun avec son titre et sa version majeure. Un artefact créé dans un parcours ajouté porte `parcours: PARCOURS-n` ; sans ce champ, il appartient au parcours par défaut. Les clarifications ne dépendent d'aucun parcours.
- Propositions en attente de décision : `.wikik/atelier/propositions/<etape>-<sous_etape>.json` (non versionné, survit au redémarrage).
- Le contexte d'un appel contient : la consigne de la sous-étape, le titre et le pitch du projet, la liste des identifiants à lire ; le LLM lit leur contenu par MCP.

## Critères d'acceptation

1. Les 11 fiches se chargent ; leurs schémas sont des JSON Schema valides ; chaque exemple respecte son schéma.
2. Avec le fournisseur factice, une séance Express de bout en bout se joue en test d'intégration et produit les fichiers attendus.
3. Après redémarrage, l'atelier reprend à la sous-étape en cours avec ses propositions.
