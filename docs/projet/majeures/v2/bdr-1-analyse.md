---
id: BDR-v2-1
type: bdr-analyse
titre: "Analyse du besoin v2 (TypeScript)"
statut: proposé
version: v2
sources: [BDR-v1-2, MESURE-01]
---

# BDR v2, partie 1 : analyse du besoin

> Aucune solution technique dans ce document.

## Contexte

La v1 de wikik ne lit que le Rust. Deux projets de Tibo sont en TypeScript (shelley-administration, shelley-studio-llm : Angular), et le wiki de wikik lui-même est écrit en TypeScript et Svelte.

## Personnes concernées

Tibo et l'agent de code, sur les projets shelley et sur le front de wikik.

## Situation actuelle et irritants

| Irritant | Preuve |
|---|---|
| Sur un projet TypeScript, l'agent n'a aucune vue ciblée : il relit les fichiers. | Les composants Angular des shelley dépassent 1 000 lignes (`vm.component.ts` : plus de 1 200). |
| Le front de wikik n'est pas couvert par sa propre doc. | EF-045 (hooks du wiki) est la seule exigence de la v1 qu'aucun `@req` ne cite : son code est en TypeScript. |
| La doc existe déjà mais n'est pas exploitée. | Les shelley ont des blocs JSDoc `/** … */` sur leurs interfaces, types et champs. |

## Contraintes

- Même outil, même index, mêmes requêtes : un projet mixte Rust + TypeScript doit fonctionner.
- Le formatage automatique (Prettier) ne doit pas rendre une doc « périmée ».

## Hors sujet

- JavaScript sans types, fichiers Svelte et gabarits Angular (HTML) : plus tard.
- Résolution par le compilateur TypeScript (types, alias `paths` de `tsconfig`).
