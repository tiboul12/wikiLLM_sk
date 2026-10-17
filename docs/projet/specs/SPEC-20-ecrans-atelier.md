---
id: SPEC-20
type: spec
titre: "Atelier : écrans"
statut: proposé
version: v4
exigences: [EF-100, EF-103, EF-109, EF-111, EF-112]
decisions: [ADR-0009, ADR-0004]
---

# SPEC-20 : écrans de l'atelier

## Choix

Même front que le wiki (Svelte 5 + TypeScript, embarqué), servi par `wikik atelier` (port 8485) qui expose aussi l'API de lecture du wiki ; l'application détecte le mode atelier par l'API `/api/atelier/etat`. Français. Navigateur (pas d'application de bureau en v4).

## Écrans

| Écran | Contenu |
|---|---|
| Tableau de bord | Stepper des 11 étapes (état, mode), consommation de l'abonnement (5 h, 7 j), accès au wiki |
| Écran générique d'étape | Sous-étapes en accordéon ; pour `propositions` : cartes garder / corriger / rejeter + « redemander » ; pour `selection` : liste à cocher ; pour `synthese` : aperçu du rapport ; indicateur « le LLM travaille » pendant un appel |
| Pitch | Formulaire en six champs, brouillon proposé par le LLM sur demande |
| Free writing | Éditeur plein écran avec minuteur (10 à 15 min), puis unités de sens extraites en cartes |
| Carte mentale | Aperçu Mermaid `mindmap` + liste éditable des branches |
| SCAMPER | Grille de 7 colonnes (un levier par colonne), cartes à garder |
| Storyboard | Planche de vignettes (description, prompt), personnage / scène / intrigue en tête |
| Bilan | Tri des besoins par glisser-déposer en catégories et priorités MoSCoW |
| Exigences | Tableau énoncé / critère / besoins / priorité, éditable |
| Roadmap | Trois colonnes Maintenant / Ensuite / Plus tard, objectif et récit par horizon |
| Clarification | Fil question / réponses, bouton « conclure » qui écrit la conclusion |

Les écrans non listés (Analogies, 5W1H, Cubing) utilisent l'écran générique en v4.

## Critères d'acceptation

Chaque écran s'affiche sur la fixture d'atelier, sans erreur de console, en clair et en sombre (contrôle en navigateur headless).
