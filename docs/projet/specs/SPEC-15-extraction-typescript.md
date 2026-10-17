---
id: SPEC-15
type: spec
titre: "Extraction TypeScript"
statut: proposé
version: v2
exigences: [EF-070, EF-071, EF-072, EF-073, EF-074]
decisions: [ADR-0002]
---

# SPEC-15 : Extraction TypeScript

## Problème

Ajouter le TypeScript derrière le trait `Langage` (SPEC-03 R1) sans changer le modèle d'index : mêmes identifiants, mêmes hash, mêmes arêtes, mêmes requêtes.

## Exigences

1. **R1** Extensions `ts`, `mts`, `cts` (grammaire TypeScript) et `tsx` (grammaire TSX) ; les `.d.ts` sont ignorés. Nom de langage `typescript`, activé par défaut avec `rust`.
2. **R2** Symboles et genres :

| Déclaration | Genre | Portée enfant |
|---|---|---|
| `function f() {}` | `fn` | — |
| `class C {}` (décorée `@Component` → `composant`) | `classe` | `C` |
| méthode de classe | `methode` | — |
| champ de classe (`public_field_definition`) | `champ` | — |
| `interface I` | `interface` | — |
| `type T = …` | `type` | — |
| `enum E` | `enum` | — |
| `const x = …` au niveau module (fonction fléchée ou expression de fonction → `fn`) | `const` / `fn` | — |
| `namespace N {}` | `mod` | `N` |

3. **R3** Visibilité : au niveau module, `export` → `pub`, sinon `prive`. Dans une classe : `private`, `protected` ou nom `#privé` → `prive`, sinon la visibilité de la classe.
4. **R4** Doc : le commentaire `/** … */` qui précède immédiatement la déclaration (ou son `export`), en sautant les décorateurs. Les `*` de début de ligne sont retirés. Résumé et balises : mêmes règles que SPEC-01.
5. **R5** En-tête de fichier : les commentaires `//` consécutifs en tête de fichier, ou un `/** … */` de tête qui contient `@file`, `@fileoverview` ou `@module`, ou qui est séparé de la déclaration suivante par une ligne vide.
6. **R6** Balises JSDoc natives acceptées sans diagnostic : `param`, `returns`, `return`, `throws`, `example`, `deprecated`, `see`, `link`, `template`, `type`, `typedef`, `default`, `remarks`, `since`, `internal`, `private`, `public`, `override`, `inheritdoc`, `file`, `fileoverview`, `module`.
7. **R7** Hash normalisé : feuilles hors commentaires, hors `;`, hors virgule finale avant `)`, `]`, `}`, `>`, hors modificateurs d'accessibilité (même raison que la visibilité Rust).
8. **R8** Fichiers `*.spec.ts`, `*.test.ts` (et `tsx`) : tous leurs symboles sont des tests.

## Proposition

### Imports

`import { a, b as c } from "./x"` → `a → <module>::a`, `c → <module>::b` ; `import D from "./d"` → `D → <module>::default` ; `import * as m from "./m"` → `m → <module>::*`. `<module>` est le chemin du fichier importé **résolu à l'extraction**, relatif à la racine, sans extension (`front/src/lib/x`) ; un import de paquet (`@angular/core`) garde son nom tel quel et n'est jamais résolu.

### Sites d'appel

| Code | Forme | Qualificatif |
|---|---|---|
| `f()` | `direct` | — |
| `this.f()` | `methode` | `self` |
| `m.f()` (m identifiant) | `methode` | `obj:m` |
| `x.y.f()` | `methode` | — |
| `new C()` | `direct` | — |
| `inject(C)` (injection Angular) | `direct` vers `C`, en plus de `inject` | — |

Les appels de l'initialiseur d'un champ (`api = inject(Api)`, `total = computed(() => …)`) sont attribués au champ : l'impact d'un service Angular liste ainsi les classes qui l'injectent.

### Résolution (ajouts à SPEC-03)

| Cas | Règle | Confiance |
|---|---|---|
| `direct f` importé depuis un module du projet | symbole de premier niveau `f` du fichier `<module>.ts`, `.tsx`, `<module>/index.ts`… | certain |
| `methode f`, qualificatif `obj:m`, `m` importé d'un module du projet (`m → <module>::*` ou `<module>::m`) | symbole de premier niveau `f` de ce module | certain |
| `direct C` sur une classe (`new C()`) | comme une fonction : les classes sont des cibles directes | certain / probable |

Le reste suit SPEC-03 (local, nom unique, méthodes courantes exclues).

## JavaScript et Svelte (v2.1)

- **JavaScript** (`js`, `jsx`, `mjs`, `cjs`, hors `.min.js`) : même extraction, grammaire TSX, langage `javascript`.
- **Svelte** : tout ce qui est hors des balises `<script>` est remplacé par des espaces (retours à la ligne gardés), puis le script passe par l'extraction TypeScript ; lignes et colonnes restent exactes. Le fichier devient un symbole `composant` nommé comme le fichier (`Cellule.svelte` → `Cellule`) : doc `<!-- @component … -->`, sinon les commentaires `//` en tête du script ; appels des instructions de premier niveau du script ; balises `<Majuscule>` du balisage en arêtes « rend », certaines si le composant est importé (`import Cellule from "./Cellule.svelte"`).
- **`wikik relu`** écrit la balise dans le format du bloc : `///`, ` * ` dans un `/** */` (ouvert sur plusieurs lignes s'il tenait sur une), ou `//`.

## Alternatives écartées

- **Compilateur TypeScript (`tsc`) pour résoudre les types** : dépendance Node, lent, hors de l'esprit « un binaire ».
- **Indexer les champs sans exiger de doc** : un champ n'est pas exigé par `check` (`manquant`), mais il est indexé pour la recherche et le wiki.

## Critères d'acceptation

1. Fixture `tests/fixtures/ts-mini` : identifiants, genres, visibilités et résumés attendus ; imports et appels résolus (EF-070, EF-071).
2. La fixture reformatée (points-virgules retirés, virgules finales ajoutées) garde tous ses hash de corps (EF-072).
3. `@param` ne produit pas de `balise-inconnue` (EF-074).
