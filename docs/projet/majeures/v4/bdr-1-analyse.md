---
id: BDR-v4-1
type: bdr-analyse
titre: "Analyse du besoin v4 (atelier de définition du besoin)"
statut: proposé
version: v4
sources: [CADRAGE-v4, SPIKE-02]
---

# BDR v4, partie 1 : analyse du besoin

> Aucune solution technique dans ce document. Matière : l'analyse du framework du 30/09/2026 (`atelier/framework/source/analyse-2026-09-30.md`) et le cadrage du 18/10.

## Contexte

Tibo a construit un framework de définition du besoin, Fast Design Thinking : 22 frameworks imbriqués (idéation, raffinage, SCAMPER, storyboards, bilan, roadmap), pilotés dans un chat par un méga-prompt de 216 Ko. wikik sait déjà relier exigences et code ; la partie amont, du pitch au cahier des charges, n'a pas d'outil.

## Personnes concernées

Tibo, qui lance ses projets (le prochain : une app de « météo » crypto / investissement) et y revient en cours de développement ; l'agent de code, qui a besoin de besoins clairs et doit pouvoir les faire préciser.

## Situation actuelle et irritants

| Irritant | Preuve |
|---|---|
| Un tiers du framework ne sert qu'à compenser le chat (état, versions, identifiants, sauvegardes). | 6 frameworks de support (Q.U.A.D., I.D.E.A., I.V.A., E.T.A.T., B.L.O.C., M.E.T.R.O.), « Formalisation & Sauvegarde » répétée 16 fois. |
| Le parcours est interminable. | Exemple réaliste : environ 45 sous-analyses, chacune validée ligne à ligne : plusieurs centaines de messages. |
| Les livrables passent d'une phase à l'autre par copier-coller. | B.I.L.A.N. demande de « refournir le dossier ou fichier consolidé » de tous les rapports. |
| Les techniques visuelles sont jouées en texte. | Carte mentale, vignettes de storyboard, grille SCAMPER. |
| Le contexte sature. | Les phases 3 à 5 reçoivent jusqu'à 8 rapports en entrée. |
| Le besoin défini ne suit pas le projet. | Une fois le chat fermé, rien ne relie l'idée d'origine à l'exigence puis au code ; on ne peut pas y revenir en plein développement. |

## Contraintes

- Pas de clé d'API à gérer : l'abonnement Claude de Tibo (CLI local) ; limites d'usage de 5 heures et 7 jours.
- Le wiki reste en lecture seule ; tout passe par les fichiers du dépôt.
- Solo en v4 ; même outil (binaire wikik).

## Hors sujet

Séances à plusieurs, application de bureau, recherche sémantique, autres langues que le français.
