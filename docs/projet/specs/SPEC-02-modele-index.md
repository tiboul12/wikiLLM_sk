---
id: SPEC-02
type: spec
titre: "Modèle de données de l'index"
statut: proposé
version: v1
exigences: [EF-001, EF-003, EF-005, EF-006, ENF-002]
decisions: [ADR-0003]
---

# SPEC-02 : Modèle de données de l'index

## Problème

Toutes les fonctions de l'outil (requêtes, check, wiki) lisent le même index. Il doit décrire le code et la zone Projet, se reconstruire à l'identique sur toute machine, et rester stable quand le code est reformaté.

## Exigences

1. **R1** Chaque symbole a un **identifiant textuel stable**, lisible, utilisable dans une URL et dans un prompt.
2. **R2** Trois **hash** distincts par symbole : corps, signature, bloc de doc.
3. **R3** Les **arêtes** portent un type et un niveau de confiance.
4. **R4** Les documents de la zone Projet sont des entités de l'index, avec leur frontmatter.
5. **R5** Le **dump JSON** est déterministe (ENF-002) : aucun identifiant interne (rowid), tri stable, aucun horodatage.
6. **R6** Le schéma porte un **numéro de version** ; un index d'une autre version est reconstruit, jamais migré en v1.

## Proposition

### Identifiants

**Symbole** : `<chemin>::<nom qualifié>[<désambiguïsation>]`
- `<chemin>` : chemin du fichier relatif à la racine du projet, séparateur `/`.
- `<nom qualifié>` : modules en ligne, type, méthode, séparés par `::`. Méthode d'un impl inhérent : `Type::methode`. Méthode d'un impl de trait : `<Type as Trait>::methode`. Les génériques sont retirés (`Foo<T>` → `Foo`) ; un trait est réduit à son dernier segment (`std::fmt::Display` → `Display`).
- `<désambiguïsation>`, seulement en cas de collision dans le fichier (SPIKE-01 : 1 cas sur 626) : `@cfg(<prédicat sans espaces>)` si l'item porte un attribut `cfg`, sinon `#2`, `#3`… dans l'ordre d'apparition.
- Exemples : `src-tauri/src/photo.rs::current`, `shared/src/lib.rs::<TextStyles as Default>::default`, `src-tauri/src/toolchain.rs::executable_candidates@cfg(windows)`.

**Fichier** : son chemin. **Document Projet** : la valeur `id` de son frontmatter si elle existe (`EF-001`, `ADR-0002`, `SPEC-01`), sinon son chemin relatif à `docs/projet/` sans extension (`plan-v1`). Les numéros d'ADR sont normalisés en `ADR-` + 4 chiffres.

**Renommage** : en v1, un symbole renommé ou déplacé est un symbole supprimé plus un symbole créé. Aucun suivi d'identité.

### Hash (SHA-256, 16 caractères hexadécimaux)

| Hash | Porte sur | Normalisation (validée par SPIKE-01) |
|---|---|---|
| `corps` | nœud `body` de l'item | feuilles de l'arbre hors commentaires ; sans virgule finale avant `)` `]` `}` `>` ; sans les accolades d'un bloc à expression unique sous un bras de `match` ou une closure ; séparateur `\0` |
| `signature` | l'item sans son corps | mêmes règles |
| `doc` | texte du bloc | lignes sans le marqueur `///`, blancs de début et de fin retirés, lignes vides finales retirées |

Pour un type, une constante ou une macro, `corps` porte sur l'item entier et `signature` est vide.

### Schéma SQLite (version 1)

```sql
CREATE TABLE meta      (cle TEXT PRIMARY KEY, valeur TEXT NOT NULL);           -- schema=1, outil=x.y.z
CREATE TABLE fichiers  (chemin TEXT PRIMARY KEY, langage TEXT NOT NULL,
                        hash_contenu TEXT NOT NULL, entete TEXT,               -- bloc //! complet
                        lignes INTEGER NOT NULL, lignes_macros INTEGER NOT NULL,
                        erreurs_parse INTEGER NOT NULL);
CREATE TABLE symboles  (id TEXT PRIMARY KEY, fichier TEXT NOT NULL REFERENCES fichiers(chemin) ON DELETE CASCADE,
                        nom TEXT NOT NULL, nom_qualifie TEXT NOT NULL, genre TEXT NOT NULL,  -- fn, methode, composant, struct, enum, trait, type, const, static, macro, mod
                        visibilite TEXT NOT NULL,                                            -- pub, pub-crate, prive
                        ligne_debut INTEGER NOT NULL, ligne_fin INTEGER NOT NULL,
                        doc_ligne_debut INTEGER,                                             -- NULL si pas de bloc
                        signature TEXT NOT NULL, resume TEXT, doc TEXT,
                        hash_corps TEXT NOT NULL, hash_signature TEXT NOT NULL, hash_doc TEXT,
                        est_test INTEGER NOT NULL);
CREATE TABLE balises   (symbole TEXT NOT NULL REFERENCES symboles(id) ON DELETE CASCADE,
                        nom TEXT NOT NULL, valeur TEXT NOT NULL, ligne INTEGER NOT NULL);
CREATE TABLE appels_bruts (symbole TEXT NOT NULL REFERENCES symboles(id) ON DELETE CASCADE,  -- sites d'appel non résolus,
                        forme TEXT NOT NULL, cible TEXT NOT NULL, qualificatif TEXT,         -- re-résolus globalement (SPEC-03)
                        ligne INTEGER NOT NULL);
CREATE TABLE imports   (fichier TEXT NOT NULL REFERENCES fichiers(chemin) ON DELETE CASCADE,
                        alias TEXT NOT NULL, chemin_import TEXT NOT NULL);
CREATE TABLE aretes    (source TEXT NOT NULL, cible TEXT NOT NULL,
                        genre TEXT NOT NULL,          -- appel, rend, adr, req, topic, lien
                        confiance TEXT NOT NULL,      -- certain, probable
                        ligne INTEGER,
                        PRIMARY KEY (source, cible, genre));
CREATE TABLE documents (id TEXT PRIMARY KEY, chemin TEXT NOT NULL UNIQUE, genre TEXT NOT NULL,  -- type du frontmatter, sinon « page »
                        titre TEXT NOT NULL, statut TEXT, version TEXT, priorite TEXT,
                        frontmatter TEXT NOT NULL,    -- JSON canonique (clés triées)
                        corps TEXT NOT NULL, hash_contenu TEXT NOT NULL);
CREATE TABLE liens_doc (source TEXT NOT NULL, cible_brute TEXT NOT NULL, cible TEXT,  -- cible NULL = lien cassé
                        ligne INTEGER NOT NULL);
CREATE VIRTUAL TABLE recherche USING fts5(cle UNINDEXED, genre UNINDEXED, titre, texte,
                        tokenize = 'unicode61 remove_diacritics 2');
```

Règles :
- Les arêtes `adr`, `req`, `topic` viennent des balises ; `lien` des liens `[[...]]` et des champs de frontmatter qui listent des identifiants (`besoins`, `exigences`, `decisions`, `sources`, `remplace`, `remplace_par`).
- La cible d'une arête peut ne pas exister (balise cassée) : c'est `check` qui le signale, l'index ne refuse rien.
- L'index vit dans `.wikik/index.sqlite`, non versionné (ADR-0003).

### Dump JSON

`wikik index --dump-json` écrit un objet `{ "schema": 1, "fichiers": [...], "symboles": [...], "aretes": [...], "documents": [...] }`. Chaque tableau est trié par sa clé primaire, chaque objet a ses clés triées, sans `meta.outil`. Ce dump est l'oracle des tests d'instantanés.

## Alternatives écartées

- **Identifiant numérique ou UUID** : illisible dans un prompt, instable entre machines.
- **Index versionné** : conflits de merge à chaque PR (décision P2).
- **Migrations de schéma** : inutiles tant que l'index se reconstruit en quelques secondes ; on reconstruit.

## Questions ouvertes

Aucune bloquante.

## Critères d'acceptation

1. Sur la fixture `tests/fixtures/rust-mini`, deux exécutions de `wikik index --dump-json`, dont une depuis une copie du dossier à un autre chemin, produisent des fichiers identiques.
2. Reformater la fixture avec `rustfmt` (`max_width = 50`) ne change aucun `hash_corps`.
3. Les identifiants de la fixture couvrent : fonction libre, méthode inhérente, méthode de trait, module en ligne, collision `cfg`.

## Exemples

| Code | Identifiant |
|---|---|
| `fn compile()` dans `src/compiler.rs` | `src/compiler.rs::compile` |
| `impl Cv { fn new() }` | `src/lib.rs::Cv::new` |
| `impl fmt::Display for AiError { fn fmt() }` | `src/ai.rs::<AiError as Display>::fmt` |
| `mod tests { fn ok() }` sous `#[cfg(test)]` | `src/lib.rs::tests::ok` (`est_test = 1`) |
| deux `fn exe()` sous `#[cfg(windows)]` et `#[cfg(not(windows))]` | `…::exe@cfg(windows)`, `…::exe@cfg(not(windows))` |
