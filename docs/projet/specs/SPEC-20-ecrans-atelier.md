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

Même front que le wiki (Svelte 5 + TypeScript, embarqué) ; l'atelier est la section « Atelier » du serveur du projet (`wikik`, ADR-0012). Français. Navigateur (pas d'application de bureau en v4).

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
| Clarification | Section Projet (EF-144) : sujet, question, pistes demandées d'office à l'ouverture ; garder / corriger / rejeter fait la réponse ; « Clarifié : terminer » la clôt, « Rouvrir », « Supprimer ». Tableau de bord `/clarifications` : clarifications rangées par sujet, avec l'état du sujet |

Les écrans non listés (Analogies, 5W1H, Cubing) utilisent l'écran générique en v4.

## API de l'atelier

Servie par le serveur du projet (`wikik`), avec l'API de lecture du wiki et celle de la réception. Les écritures sont en `POST` JSON ; une requête dont l'en-tête `Origin` ne correspond pas à l'hôte est refusée (403), un corps non JSON aussi (415).

| Route | Corps | Effet |
|---|---|---|
| `GET /api/atelier/etat?parcours=` | | pour un parcours (EF-145, `PARCOURS` par défaut) : étapes (fiche, état, mode, artefacts, sélections, lots), artefacts, clarifications, fournisseur, consommation, liste des parcours |
| `POST /api/atelier/parcours` | `titre`, `version` | nouveau parcours `PARCOURS-n` |
| `POST /api/atelier/clarification` | `id`, `action` (`clore`, `rouvrir`, `supprimer`) | clarification close (pistes gardées en conclusion), rouverte ou supprimée |
| `POST /api/atelier/piloter` | `etape`, `action` (`ouvrir`, `terminer`, `sauter`, `rouvrir`, `mode`), `mode` | état ou mode d'une étape |
| `POST /api/atelier/selection` | `etape`, `sous_etape`, `ids` | sélection d'une sous-étape `selection` |
| `POST /api/atelier/proposer` | `etape`, `sous_etape`, `cible`, `entree` | appel au LLM, propositions ajoutées au lot |
| `POST /api/atelier/decider` | `etape`, `sous_etape`, `cible`, `indice`, `decision` (`garder`, `rejeter`), `donnees` (correction) | artefact créé ou complété |
| `POST /api/atelier/saisir` | `etape`, `sous_etape`, `cible`, `donnees` | pitch, texte libre, tri des besoins |
| `POST /api/atelier/synthetiser` | `etape`, `sous_etape` | rapport assemblé ; renvoie les fichiers écrits |
| `POST /api/atelier/rappel` | `sujet`, `question` (facultative) | clarification `CLAR-xxx` ; sans question, celle qui est ouverte sur le sujet est reprise |

Les écritures d'un parcours (`piloter`, `selection`, `proposer`, `decider`, `saisir`, `synthetiser`) portent `parcours` (défaut `PARCOURS`).

## Critères d'acceptation

Chaque écran s'affiche sur la fixture d'atelier, sans erreur de console, en clair et en sombre (contrôle en navigateur headless).
