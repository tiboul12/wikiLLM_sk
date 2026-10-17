---
id: BDR-v3-2
type: bdr-definition
titre: "Définition du besoin v3 (équipe)"
statut: proposé
version: v3
sources: [BDR-v3-1]
---

# BDR v3, partie 2 : définition du besoin

## Objectifs mesurables

1. Le wiki d'un dépôt se lit dans un navigateur à partir d'un dossier statique, sans serveur wikik, recherche et graphe compris.
2. Une PR qui casse une référence (`@req`, `@adr`, lien) est signalée sur la ligne concernée et échoue en CI.
3. Un nouvel identifiant (`EF`, `B`, `ADR`, `SPEC`) se crée en une commande, sans collision locale.
4. Les documents produit d'une version majeure ont des validateurs désignés dans le dépôt.

## Périmètre

| Inclus | Exclu |
|---|---|
| Export statique, CI GitHub, annotations, CODEOWNERS, hook pre-commit, création d'identifiants, statistiques | Outil de tickets, portail multi-dépôts, WASM |

## Besoins

| ID | Besoin |
|---|---|
| [[besoins/B-011]] | Partager le wiki et ses garde-fous avec une équipe, sans serveur à maintenir. |

B-008 (passer en équipe sans tout refaire) est le besoin d'origine.

## Critères de succès

Les objectifs ci-dessus vérifiés sur wikiK : export ouvert dans un navigateur headless, annotations au format GitHub, modèle de CI généré.

## Décision de lancement

**Go**, le 17/10/2026, par délégation de Tibo (exécution en continu demandée) : à confirmer à sa relecture.
