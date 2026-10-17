---
id: SPEC-03
type: spec
titre: "Extraction et indexation incrémentale"
statut: proposé
version: v1
exigences: [EF-001, EF-002, EF-004, EF-005, EF-007, EF-029, ENF-003, ENF-006]
decisions: [ADR-0002, ADR-0003]
---

# SPEC-03 : Extraction et indexation incrémentale

## Problème

Transformer les fichiers du projet en index (SPEC-02), vite, sans jamais bloquer sur un fichier mal formé, et en ne refaisant que ce qui a changé.

## Exigences

1. **R1** Un module par langage derrière un trait commun ; ajouter TypeScript ne touche pas au reste.
2. **R2** Seuls les fichiers dont le hash de contenu a changé sont ré-analysés ; les fichiers disparus sont retirés.
3. **R3** La résolution des appels est **globale** et refaite à chaque indexation, à partir des sites d'appel bruts stockés par fichier : un fichier modifié peut changer la cible d'un appel d'un autre fichier.
4. **R4** Un fichier qui ne se parse pas entièrement est indexé partiellement et compté (`erreurs_parse`), jamais bloquant.
5. **R5** Toute requête vérifie la fraîcheur de l'index et le met à jour d'abord (EF-029).
6. **R6** Cibles de vitesse (ENF-003) : premier index de 50 000 lignes < 2 s ; mise à jour d'un fichier < 200 ms ; requête < 100 ms.

## Proposition

### Parcours des fichiers

- Crate `ignore` : respecte `.gitignore`, ignore `target/`, `.git/`, `.wikik/`, `node_modules/`, plus les motifs `exclure` de `wikik.toml`.
- Code : fichiers dont l'extension appartient à un langage activé (`.rs` en v1).
- Zone Projet : tous les `.md` sous `docs/projet/` (chemin réglable : `projet.dossier`).

### Trait de langage

```rust
pub trait Langage {
    fn nom(&self) -> &'static str;                 // "rust"
    fn extensions(&self) -> &'static [&'static str];
    fn extraire(&self, chemin: &str, source: &[u8]) -> ExtraitFichier;
}
// ExtraitFichier = en-tête, symboles (avec doc, balises, hash), sites d'appel bruts,
// imports, lignes en macros, nombre d'erreurs de parse.
```

L'extraction Rust **parcourt l'arbre tree-sitter à la main** (comme le spike) plutôt que par requêtes `.scm` : il faut suivre la portée (modules, impl, traits) pour construire les identifiants, ce qu'un parcours exprime plus simplement. Le plan mentionnait des `.scm` : abandonné pour cette raison.

### Extraction Rust (règles validées par SPIKE-01)

- **Items** : `function_item`, `function_signature_item` (méthodes de trait), `struct_item`, `enum_item`, `union_item`, `trait_item`, `type_item`, `const_item`, `static_item`, `macro_definition`, `mod_item` en ligne (portée), `impl_item` (portée).
- **Doc et attributs** : frères précédents contigus, `///` (pas `////`), `/** */`, et `attribute_item` intercalés. `#[component]` → genre `composant`. `#[cfg(...)]` → prédicat retenu pour la désambiguïsation. `#[test]` ou portée `#[cfg(test)] mod` → `est_test`.
- **En-tête** : lignes `//!` en tête de fichier.
- **Visibilité** : `visibility_modifier` `pub` → `pub` ; `pub(crate)`, `pub(super)`, `pub(in …)` → `pub-crate` ; sinon `prive`. Méthode d'un impl de trait : visibilité du type n'est pas calculée en v1, on la classe `pub` si le trait est implémenté pour un type `pub` du même fichier, sinon `prive`.
- **Imports** : chaque `use_declaration` est aplatie en paires `alias → chemin complet` (`use a::{b, c as d}` → `b → a::b`, `d → a::c`).
- **Sites d'appel** (corps de fonction) : `call_expression` avec forme `direct` (identifiant), `chemin` (`a::b::f`), `methode` (`x.f()`, qualificatif `self` si le receveur est `self`), `generique`. Composants Leptos : dans une `macro_invocation` `view!`, un `<` suivi d'un identifiant en majuscule → forme `rend`.
- **Lignes en macros** : somme des lignes des `macro_invocation`, pour signaler la part non analysée.

### Résolution des appels (globale, à chaque indexation)

Pour chaque site d'appel brut, dans l'ordre, la première règle qui s'applique :

| Forme | Règle | Confiance |
|---|---|---|
| `direct` `f` | fonction `f` au niveau module dans le même fichier | certain |
| `direct` `f` | `f` importé par `use` vers un chemin dont le suffixe désigne un symbole unique du projet | certain |
| `direct` `f` | un seul symbole fonction nommé `f` dans le projet | probable |
| `chemin` `Self::f` / `T::f` | dans un impl de `T` : `T::f` du même fichier ; sinon `T::f` unique dans le projet | certain |
| `chemin` `a::b::f` | suffixe de module qualifié unique (`b::f`, puis `f` dans un fichier `b.rs` ou `b/mod.rs`) | certain si unique, sinon probable vers au plus 3 candidats |
| `methode` `self.f()` | dans un impl de `T` : `T::f` ou `<T as _>::f` du même type | certain |
| `methode` `x.f()` | `f` est une méthode unique du projet et n'est pas dans la liste des noms courants de la bibliothèque standard (`clone`, `unwrap`, `into`, `iter`, `map`, `len`, `push`, `get`, `set`, `new`, `to_string`, `as_ref`…) | probable |
| `rend` `<C>` | composant unique nommé `C` | probable |
| autre | non stocké ; compté dans les statistiques du fichier | — |

Un appel vers soi-même (récursion) est stocké. Les doublons (même source, cible, genre) sont fusionnés en gardant la confiance la plus forte et la première ligne.

### Zone Projet

- Frontmatter : bloc YAML entre deux lignes `---` en tête de fichier. Parse en `serde_json::Value` puis JSON canonique. Frontmatter invalide → document indexé avec `genre = page` et diagnostic `frontmatter-invalide` (SPEC-04).
- Titre : `titre` du frontmatter, sinon premier `# Titre`, sinon nom de fichier.
- Liens : `[[cible]]` et `[[cible|libellé]]` hors blocs de code ; résolution par identifiant de document, puis par chemin relatif à `docs/projet/` (avec ou sans `.md`), puis par nom de fichier unique. Ancre `#section` conservée mais non vérifiée en v1.
- Champs de liens du frontmatter : `besoins`, `exigences`, `decisions`, `sources`, `remplace`, `remplace_par` → arêtes `lien`.

### Algorithme incrémental

```text
1. Ouvrir .wikik/index.sqlite ; si absent ou schema ≠ 1 → le recréer vide.
2. Lister les fichiers (code + docs). Pour chacun, hash du contenu (SHA-256).
3. Fichiers absents de la liste → DELETE (cascade sur symboles, balises, appels bruts, imports).
4. Fichiers nouveaux ou de hash différent → extraire puis remplacer leurs lignes, dans une transaction.
5. Si au moins un fichier a changé : recalculer toutes les arêtes `appel`/`rend` (R3) et les arêtes de balises et de liens ; reconstruire la table `recherche`.
6. Écrire meta.dernier_index = liste triée des (chemin, hash) → sert au test de fraîcheur.
```

Test de fraîcheur (R5) : relister les fichiers et comparer taille + date de modification au cache ; ne recalculer le hash que pour les fichiers dont la taille ou la date a changé. Coût attendu : quelques millisecondes.

## Alternatives écartées

- **Requêtes tree-sitter `.scm`** : voir plus haut.
- **Résolution des appels par fichier** : fausse dès qu'un autre fichier change (R3).
- **rust-analyzer / analyse de types** : précis mais lourd, lent et spécifique à Rust ; hors esprit « un binaire, plusieurs langages ».

## Questions ouvertes

Aucune bloquante. La liste des noms de méthodes courants s'ajustera à l'usage sur ATS.

## Critères d'acceptation

1. Fixture : chaque règle de résolution du tableau a un cas de test.
2. Modifier un seul fichier de la fixture : l'index ne ré-analyse que celui-ci (compteur `analysés: 1`), et son dump est identique à celui d'une reconstruction complète.
3. Une fixture avec erreur de syntaxe s'indexe ; `erreurs_parse > 0`.
4. ATS : premier index < 500 ms ; mesure consignée.
