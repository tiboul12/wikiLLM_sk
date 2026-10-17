---
id: SPEC-01
type: spec
titre: "Convention de blocs de doc dans le code"
statut: proposé
version: v1
exigences: [EF-006, EF-011, EF-030]
decisions: [ADR-0002]
---

# SPEC-01 : Convention de blocs de doc dans le code

## Problème

L'humain et l'agent doivent écrire la doc de la même façon, à un seul endroit (le code), et l'outil doit pouvoir la lire sans ambiguïté. Les trois outils de septembre donnaient des règles différentes (docstrings, bloc ADR en commentaire, fiches dans `wiki/`).

## Exigences

1. **R1** La doc d'un symbole est un **commentaire de doc natif** du langage. En Rust : `///` avant l'item, `//!` en tête de fichier. Aucune syntaxe propriétaire (pas de `@wiki … @end`).
2. **R2** La **première phrase** du premier paragraphe d'un bloc est le **résumé** (elle peut courir sur deux lignes ; 120 caractères conseillés, coupée à 200). C'est elle qu'affichent `carte`, `fichier` et les listes du wiki.
3. **R3** Le reste du bloc dit ce que le code ne dit pas : **intention, contrat** (entrées admises, erreurs, effets de bord, `panic`), **pièges**, **raison d'un choix**. Il ne paraphrase pas le code.
4. **R4** Les **balises** sont en fin de bloc, une par ligne, en début de ligne : `@adr`, `@req`, `@topic`, `@pitfall`, `@relu`. Liste fermée en v1 ; une balise inconnue produit un avertissement. Dans un en-tête `//!`, une balise vaut pour **tout le fichier** (la source de l'arête est le chemin du fichier).
5. **R5** Doivent avoir un bloc : les items `pub` (fonctions, méthodes, types, traits, constantes, macros exportées). Les autres sont libres. Les tests (`#[test]`, `mod tests` sous `#[cfg(test)]`) sont exclus. Seuil réglable dans `wikik.toml` (`pub`, `pub-crate`, `tout`).
6. **R6** Chaque fichier devrait avoir un **bloc d'en-tête** `//!` : rôle du fichier en une ligne, puis entrées, sorties, choix de conception. Manquant = information, pas avertissement.
7. **R7** Les références vers d'autres symboles s'écrivent en **lien intra-doc** Rust (`` [`Type::methode`] ``). Le préfixe `@wiki:` est réservé aux **références copiées dans un prompt** (hooks du wiki), pas aux blocs.

## Proposition

### Forme d'un bloc

```rust
/// Compile `template` dans `work_dir`, puis copie le PDF vers `final_pdf_path`.   ← résumé (R2)
///
/// `work_dir` est vidé avant chaque passe : un `.aux` résiduel d'une passe          ← contrat, piège (R3)
/// non convergée casse la reproductibilité.
///
/// @req EF-012                                                                     ← balises (R4)
/// @adr 3
/// @pitfall xelatex absent du PATH : renvoie une erreur, ne panique jamais.
pub fn compile(/* … */) { /* … */ }
```

### Grammaire des balises

| Balise | Arguments | Normalisation | Effet dans l'index |
|---|---|---|---|
| `@adr` | un ou plusieurs numéros d'ADR, séparés par `,` ou espace : `3`, `0003`, `ADR-0003` | `ADR-0003` | arête `adr` symbole → ADR |
| `@req` | un ou plusieurs `EF-xxx` / `ENF-xxx` | majuscules | arête `req` symbole → exigence |
| `@topic` | un ou plusieurs identifiants courts en kebab-case | minuscules | arête `topic` ; page transversale dans le wiki |
| `@pitfall` | texte libre jusqu'à la fin de la ligne | aucune | attribut « piège », mis en avant dans `symbole` et `impact` |
| `@relu` | date `AAAA-MM-JJ`, puis jeton de 8 caractères écrit par `wikik relu` | aucune | déclare le bloc relu pour le code dont le jeton est l'empreinte (voir SPEC-04) |

Règle de reconnaissance : après `///` ou `//!` et des blancs, la ligne commence par `@` suivi d'un nom de balise en minuscules. Un `@` au milieu d'une phrase (adresse, mention) n'est jamais une balise.

### Critères vérifiables de « ne paraphrase pas le code » (R3)

`wikik check` émet une **information** `paraphrase` quand :
- le résumé, une fois retirés les mots vides (« le », « la », « de », « renvoie », « retourne », « the », « returns »…), ne contient que des mots présents dans le nom du symbole, ses paramètres et ses types ;
- ou le bloc dépasse 25 lignes (`bloc-long`), signe qu'il raconte le code au lieu de le cadrer.

Ce sont des indices pour la relecture, jamais des erreurs.

## Alternatives écartées

- **Syntaxe propre `@wiki … @end`** : double convention, ignorée par rustdoc et les IDE.
- **Bloc ADR complet dans le code** (skill `doc-as-code`) : l'ADR devient illisible dès qu'il dépasse cinq lignes et ne se relie pas à plusieurs endroits du code. On garde l'ADR en fichier (`docs/projet/adr/`) et une balise `@adr` dans le code.
- **Dépendances écrites à la main dans le bloc** : elles sont calculées (décision D3).

## Questions ouvertes

Aucune bloquante. La liste des mots vides de l'heuristique `paraphrase` s'ajustera à l'usage.

## Critères d'acceptation

Les 10 exemples ci-dessous, tirés d'ATS, sont classés bon / à revoir ; Tibo valide ce classement (par délégation le 17/10, à confirmer).

## Exemples

| # | Bloc (ATS) | Verdict | Pourquoi |
|---|---|---|---|
| 1 | `compiler.rs::compile` : « Compile `template` dans `work_dir`, puis copie le PDF… `data_overrides`, si fourni, est écrit tel quel dans `cvdata.tex` … le template doit lui-même faire `\InputIfFileExists` » | ✅ Bon | Résumé + contrat implicite que le code ne montre pas (ce que le template doit faire). |
| 2 | `photo.rs::set_from_file` : « Seuls les vrais JPEG/PNG sont acceptés (contrôle du contenu, pas de l'extension) : ce sont les formats que XeLaTeX sait inclure. » | ✅ Bon | Contrat + raison du choix. |
| 3 | `photo.rs::extract_from_cv` : « `Ok(false)` : aucune image convaincante trouvée (la photo existante est conservée). » | ✅ Bon | Sens d'une valeur de retour, effet de bord explicite. |
| 4 | `photo.rs::image_size` : « … lues dans l'en-tête — pas de dépendance à une bibliothèque d'images pour si peu. » | ✅ Bon | Raison d'un choix d'implémentation. |
| 5 | `lib.rs::flow_paragraphs` : « Découpe un texte en paragraphes **linéaires**, comme le fait LaTeX … L'aperçu et le PDF s'appuient sur cette même fonction. » | ✅ Bon | Règle métier + avertissement implicite (deux usages) ; pourrait porter `@pitfall`. |
| 6 | `app.rs::import` (ligne 1353) : « … voir wiki/decisions/007-import-word-reprio.md et 009-ai-cli-integration.md » | 🟡 À convertir | Bon fond ; la référence en texte libre devient `@adr 7, 9` pour être vérifiée et reliée. |
| 7 | `toolchain.rs` (en-tête) : « Détection des toolchains externes requises par le pipeline (AC-009, F1, F2). » | 🟡 À convertir | Les identifiants d'exigences existent déjà en texte libre ; ils deviennent `@req` une fois les exigences d'ATS au format `EF-xxx`. |
| 8 | `photo.rs::current` : « Chemin de la photo enregistrée, s'il y en a une. » | 🟡 Limite | Paraphrase presque la signature `Result<Option<PathBuf>>` ; acceptable pour une fonction triviale, mais n'apporte rien sur l'erreur `String`. |
| 9 | `app.rs::textarea_field` : « Champ texte multi-lignes (descriptions). » | 🟡 Limite | Résumé correct, aucun contrat ; suffisant pour un composant d'interface simple. |
| 10 | Contre-exemple construit : « Renvoie le nom de l'utilisateur. » sur `fn user_name(&self) -> String` | ❌ Mauvais | Paraphrase pure : tous les mots sont dans la signature. Déclenche `paraphrase`. |
