---
id: SPEC-17
type: spec
titre: "Atelier : modèle de données"
statut: proposé
version: v4
exigences: [EF-105, EF-106, EF-107, EF-108]
decisions: [ADR-0010]
---

# SPEC-17 : modèle de données de l'atelier

## Problème

Chaque production de l'atelier doit être un fichier lisible par le wiki et l'agent, et garder le lien avec ce dont elle vient (B-014).

## Exigences

1. **R1** Un fichier Markdown par artefact sous `docs/atelier/`, avec frontmatter (`id`, `type`, `titre`, `statut`, `technique`, `sources`, champs propres au type). Le corps reprend le contenu lisible.
2. **R2** Identifiants : préfixe par type, numéro suivant libre sur 3 chiffres, jamais réutilisé (même règle que `wikik nouveau`).
3. **R3** `sources` liste les identifiants dont l'artefact découle ; c'est la traçabilité (arêtes `lien` de l'index).
4. **R4** Statuts des artefacts : `retenu` (gardé à l'écran), `ecarte` (gardé pour mémoire, non utilisé), `brouillon` (en cours d'écriture).
5. **R5** Ce qui sort vers le développement (besoins, BDR, exigences, roadmap) est écrit dans `docs/projet/` avec les gabarits existants, statut `proposé`, et cite ses sources de l'atelier.

## Types d'artefacts

| Type | Préfixe / fichier | Produit par | Champs propres |
|---|---|---|---|
| `pitch` | `PITCH` · `pitch.md` | Pitch | `probleme`, `cible`, `valeur`, `contraintes`, `criteres_succes`, `hors_sujet` |
| `axe` | `AXE-` · `axes/` | Free writing (cadrage, C.I.S.) | `description` |
| `ecrit` | `FW-` · `ecrits/` | Free writing (action, P.A.R.S.) | `axe`, `duree_minutes` ; corps = texte brut, jamais modifié |
| `unite` | `US-` · `unites/` | Free writing (récolte) | `qualification` (idée, problème, émotion, frustration…), `axe` ; corps = verbatim |
| `carte` | `CARTE` · `carte.md` | Carte mentale | corps = `mindmap` Mermaid ; `noeuds` (identifiants) |
| `probleme` | `PB-` · `problemes/` | Analogies (C.I.L.) | `origine` |
| `analogie` | `ANL-` · `analogies/` | Analogies (P.O.N.T.) | `probleme`, `domaine`, `principes` |
| `idee` | `IDEE-` · `idees/` | Free writing, Carte, Analogies, SCAMPER | `description`, `technique` |
| `analyse` | `ANA-` · `analyses/` | 5W1H (C.E.S.), Cubing (C.A.R.) | `methode` (`5w1h` ou `cubing`), `sujet` ; une section par question ou face |
| `scamper` | `SCA-` · `scamper/` | SCAMPER | `sujet`, `idees` par levier |
| `storyboard` | `SB-` · `storyboards/` | Storyboard (R.E.C.I.T.) | `scenario`, `personnage`, `scene`, `intrigue`, `vignettes` (id, description, prompt), `artefacts` (id `ART-`, nom, vignette) |
| `clarification` | `CLAR-` · `clarifications/` | Rappel en développement | `sujet` (identifiant visé), `question`, `conclusion`, `etat` (`ouverte`, `close`) |
| `parcours` | `PARCOURS` · `parcours.md` (ou `PARCOURS-n` · `parcours-n/parcours.md`) | Moteur | `titre`, `version`, `mode`, état de chaque étape, sous-étape courante |

Vers `docs/projet/` : `besoin` (`B-`, champs `categorie`, `priorite` MoSCoW, `sources`), BDR 1 et 2, `exigence` (`EF-`), `roadmap` (`majeures/v1/roadmap.md`, horizons Maintenant / Ensuite / Plus tard).

## Index et contrôles

- L'index lit `docs/atelier/` comme une seconde zone de documents (champs de liens : `sources`, `sujet`, `probleme`, `axe`, plus ceux de la zone Projet).
- `check`, seulement si le projet a un dossier d'atelier : `exigence-sans-besoin` et `besoin-sans-source` (avertissements). Un projet sans atelier (ATS aujourd'hui) n'est pas concerné.

## Critères d'acceptation

1. Une idée retenue en SCAMPER a pour `sources` l'idée analysée ; la page du wiki de l'exigence finale remonte jusqu'à elle en suivant les liens.
2. Deux idées créées à la suite reçoivent `IDEE-001` puis `IDEE-002`, même si `IDEE-001` est ensuite écartée.
