---
name: wikik-clarification
description: Proposer des pistes pour lever un flou sur un besoin, une exigence ou un élément de l'atelier, en jugeant d'abord si la question porte vraiment sur le sujet. Utilisé par l'écran Clarification de wikik et par l'outil atelier_rappel.
---

# Clarifier un sujet

Une clarification lève un flou sur un **sujet** (besoin, exigence, storyboard, symbole) : le LLM lit le sujet, ses sources et le projet, puis propose des pistes ; l'utilisateur garde, corrige ou rejette chacune, et les pistes gardées font la réponse.

## 1. Juger la question

Avant toute piste, décider si la question porte sur le sujet :

- **Sur le sujet** : elle interroge le contenu du sujet (ce qu'il demande, pour qui, ses limites, ses critères).
- **Hors sujet** : elle porte sur autre chose, par exemple sur l'outil lui-même (« à quoi sert cette fenêtre ? »), sur un autre document, ou elle est vide de sens pour ce sujet.

Si elle est hors sujet, le dire franchement dans `hors_sujet` (une phrase : pourquoi, et où poser la question), puis proposer quand même au plus une piste qui ramène au sujet.

## 2. Proposer des pistes

- Deux à quatre pistes, chacune avec sa justification et ses sources (identifiants exacts : `B-001`, `EF-144`, `chemin::Symbole`, route d'écran `/clarifications`).
- Citer les écrans de l'application par leur route et les fichiers par leur chemin : l'écran en fait des liens.
- Signaler une contradiction entre documents plutôt que de la masquer.
- Ne rien inventer : ce qui n'est pas dans le projet n'est pas une source.

## 3. Style

Français, phrases courtes, une idée par piste.
