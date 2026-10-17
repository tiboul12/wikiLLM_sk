---
id: SPEC-07
type: spec
titre: "Requêtes pour l'agent (CLI)"
statut: proposé
version: v1
exigences: [EF-020, EF-021, EF-022, EF-023, EF-024, EF-025, EF-026, EF-027, EF-028, EF-029, ENF-001, ENF-003]
decisions: [ADR-0003]
---

# SPEC-07 : Requêtes pour l'agent

## Problème

L'agent doit obtenir le contexte utile d'un projet sans ouvrir les fichiers source, avec un coût en tokens prévisible (ENF-001). Les requêtes sont la seule interface agent de la v1 ; le futur serveur MCP exposera les mêmes.

## Exigences

1. **R1** Chaque requête met d'abord l'index à jour (incrémental, EF-029).
2. **R2** Sortie **texte compacte** par défaut, pensée pour un prompt ; `--format json` pour les outils.
3. **R3** La première ligne de chaque sortie texte commence par `#` et annonce l'objet et le **coût estimé** (`~N tokens`, octets / 4).
4. **R4** Le code n'est jamais renvoyé sans le demander (`--code`).
5. **R5** Un identifiant de symbole partiel est accepté s'il désigne un seul symbole (`norme`, `Point::norme`) ; sinon la sortie liste les candidats.
6. **R6** Les arêtes affichent leur confiance quand elle n'est pas `certain`.

## Proposition

| Requête | Contenu | Budget (ENF-001) |
|---|---|---|
| `carte` | En-tête : nombre de fichiers, symboles, taux de doc, nombre de périmés. Puis par dossier : fichiers avec la première phrase de leur en-tête `//!` et leurs compteurs. Puis la zone Projet par type de document. Au-delà de 150 fichiers : dossiers seulement, `--complet` pour tout. | ≤ 4 000 |
| `fichier <chemin>` | En-tête `//!` complet, imports (aliases), symboles (ligne, visibilité, genre, nom, résumé, statut de doc), part de code en macros. | ≤ 1 500 |
| `symbole <id>` | Signature, doc complète, statut (à jour / périmé / manquant), appelle, appelé par, ADR et exigences liés avec leur titre, pièges. `--code` ajoute le code. | ≤ 800 sans code |
| `impact <id>` | Appelants transitifs par distance (défaut 3), puis ADR, exigences et pièges de l'ensemble. | ≤ 1 500 |
| `perimes` | Symboles à la doc périmée ou manquante : `fichier:ligne code symbole`. | proportionnel |
| `cherche <texte>` | 15 meilleurs résultats FTS5 (bm25) sur blocs et documents, avec résumé et extrait. | ≤ 1 500 |
| `adr <id>` | Titre, statut, section « Décision » (600 caractères), symboles et documents qui le citent. | ≤ 800 |
| `exigence <id>` | Titre, statut, priorité, besoins, énoncé, critère d'acceptation, symboles et tests qui la citent, specs qui la couvrent. | ≤ 800 |
| `ref <@wiki:…>` | Résout `@wiki:[projet/]chemin::Symbole` (→ `symbole`), `@wiki:[projet/]chemin.rs` (→ `fichier`), `@wiki:[projet/]doc[#ancre]` (→ document). | selon la cible |

Statut de doc d'un symbole : calculé comme `check` (SPEC-04), mais seulement pour les fichiers concernés par la requête.

## Alternatives écartées

- **Sortie Markdown riche** (tableaux) : plus de tokens pour la même information.
- **Code inclus par défaut** : c'est précisément ce qu'on veut éviter de lire.

## Critères d'acceptation

1. Sur ATS : `carte` ≤ 4 000 tokens estimés, `fichier src/app.rs` ≤ 1 500, `symbole` ≤ 800, `impact` ≤ 1 500 ; mesures consignées dans `docs/projet/mesures/`.
2. Sur la fixture : `impact src/geo.rs::Point::carre` liste `Point::norme` (distance 1) et `entree` (distance 2, probable).
3. `symbole norme` est résolu sans le chemin ; `symbole exe` liste les deux variantes `cfg`.
