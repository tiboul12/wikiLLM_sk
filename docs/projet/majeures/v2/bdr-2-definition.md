---
id: BDR-v2-2
type: bdr-definition
titre: "Définition du besoin v2 (TypeScript)"
statut: proposé
version: v2
sources: [BDR-v2-1]
---

# BDR v2, partie 2 : définition du besoin

## Objectifs mesurables

1. Sur shelley-studio-llm, `wikik query carte` et `wikik query fichier` décrivent le front Angular sans ouvrir de fichier.
2. Reformater un fichier TypeScript avec Prettier ne change aucun hash de corps.
3. Toutes les exigences `must` de la v1 **et** EF-045 sont citées par du code ou un test.

## Périmètre

| Inclus | Exclu |
|---|---|
| Fichiers `.ts`, `.tsx`, `.mts`, `.cts` (hors `.d.ts`) | JavaScript, `.svelte`, gabarits Angular |
| JSDoc `/** */`, balises wikik, balises JSDoc natives | Résolution des alias `tsconfig` (`@shelley/...`) |
| Imports relatifs, appels directs, `this.f()`, `module.f()` | Inférence de types |

## Besoins

| ID | Besoin |
|---|---|
| [[besoins/B-010]] | Documenter et interroger les projets TypeScript comme les projets Rust. |

Les besoins B-001 à B-009 restent valables pour TypeScript.

## Critères de succès

Ceux des objectifs ci-dessus, vérifiés sur la fixture TypeScript, sur le front de wikik et sur une copie de shelley-studio-llm.

## Décision de lancement

**Go**, le 17/10/2026, par délégation de Tibo (exécution en continu demandée) : à confirmer à sa relecture.
