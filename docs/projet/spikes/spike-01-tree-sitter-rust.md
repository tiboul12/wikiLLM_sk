---
id: SPIKE-01
type: spike
statut: terminé
version: v1
date: 2026-10-17
sources:
  - docs/projet/plan-v1.md
---

# SPIKE-01 : tree-sitter sur du Rust réel (ATS)

**But** : vérifier, avant SPEC-02 et SPEC-03, les hypothèses dont dépend le modèle d'index.
**Code** : `spikes/spike-ts/` (jetable, non versionné). `tree-sitter 0.27.1` + `tree-sitter-rust 0.24.2`, SHA-256.
**Terrain** : ATS, 29 fichiers, 11 163 lignes, workspace de 3 crates (Leptos + Tauri). Vitesse mesurée aussi sur les sources du registre cargo (35 904 fichiers, 15,3 M lignes).

## Résultats

| # | Question | Résultat | Conséquence |
|---|---|---|---|
| S1 | Identifiant stable | Forme `chemin/fichier.rs::Type::methode` ; impl de trait en `<Type as Trait>::methode`. Sur ATS : **626 symboles, 1 collision** (`executable_candidates`, deux variantes `#[cfg(windows)]` / `#[cfg(not(windows))]`). Sur 15 M lignes : 0,4 % de collisions (variantes `cfg`, code généré). | Garder cette forme. Désambiguïser les collisions par un suffixe : attribut `cfg` s'il existe (`fn@cfg(windows)`), sinon ordinal (`#2`). Retirer les génériques du type (`Foo<T>` → `Foo`) ; ramener le trait à son dernier segment (`std::fmt::Display` → `Display`). |
| S2 | Hash du corps | Hash des feuilles hors commentaires. Premier essai : 31 corps sur 107 changent après un rustfmt agressif (`max_width = 50`). Causes : virgules finales ajoutées (78 cas), accolades autour d'un bras de `match` ou d'un corps de closure à expression unique. Après normalisation de ces deux cas : **0 changement après reformatage, 0 changement quand seuls les `///` ou les commentaires changent, une vraie modification (`40.0` → `41.0`) est détectée.** | Hash = feuilles du corps, hors commentaires, sans virgule finale avant un délimiteur fermant, sans accolades d'un bloc à expression unique sous `match_arm` / `closure_expression`. Hash de signature séparé (item sans corps ni doc). |
| S3 | Blocs de doc | Les `///` et les attributs qui les suivent (`#[cfg]`, `#[component]`, `#[derive]`) se rattachent de façon fiable à l'item en remontant les frères précédents. ATS : 237 / 626 symboles documentés (198 / 483 fonctions), 219 lignes `//!`. Aucune balise `@` encore. | Rattachement par frères précédents (doc puis attributs). Le `//!` de tête de fichier = bloc d'en-tête. ATS part d'une base déjà documentée à 38 % : l'amorçage est plus léger que prévu. |
| S4 | Appels | 4 578 appels. Formes : méthode `x.m()` 61 %, direct `f()` 21 %, chemin `a::b()` 17 %, générique 1 %. Par nom court : 19 % visent une seule fonction du projet, 4 % plusieurs candidates, 77 % aucune (bibliothèque standard, crates externes, méthodes). | Sans analyse de types, seuls les appels directs et par chemin se résolvent proprement. Trois niveaux d'arête : **certain** (nom défini dans le même fichier ou chemin résolu par les `use`), **probable** (nom unique dans le projet, y compris méthodes), **inconnu** (non stocké en v1). Le graphe affiche le niveau. |
| S5 | Macros | 24,4 % des lignes d'ATS sont dans des invocations de macro, dont **15 % dans `view!`** (Leptos). Leur contenu est un arbre de tokens non analysé. Mais une heuristique simple y retrouve 67 balises `<Majuscule …>`, dont **52 composants du projet**, et 901 motifs `ident(…)`. | Extraire de `view!` les références `<Composant>` vers les composants connus (arête « probable », type `rend`). Ignorer les autres macros en v1, et signaler dans `fichier` la part de code non analysée. |
| S6 | Vitesse | ATS : **172 ms** pour tout analyser. 15,3 M lignes : 249 s sur un seul cœur, soit **16 µs par ligne**. 4,5 % des fichiers du registre ont une erreur de parse (syntaxe récente, fixtures invalides) : tree-sitter rend quand même un arbre partiel. | Cibles : premier index d'un projet de 50 000 lignes < 2 s sur un cœur ; ré-analyse d'un fichier < 50 ms. Parallélisme par fichier si besoin (non nécessaire en v1). Un fichier avec erreur de parse est indexé partiellement et signalé, jamais bloquant. |

## Ce que ça change au plan

- **Aucune hypothèse de la roadmap n'est invalidée** : tree-sitter suffit pour les symboles, la doc et le hash ; il est rapide.
- **SPEC-02** gagne : l'identifiant avec suffixe de désambiguïsation, trois hash (corps, signature, bloc), trois niveaux d'arête.
- **SPEC-03** gagne : les règles de normalisation du hash (testées), l'heuristique `view!`, la tolérance aux erreurs de parse, les cibles de vitesse chiffrées.
- **Risque « macros » réduit** : la partie visible des composants Leptos est récupérable ; le reste est signalé honnêtement.
