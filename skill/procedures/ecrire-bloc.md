# Écrire ou corriger un bloc de doc

Référence : SPEC-01 (`docs/projet/specs/SPEC-01-convention-blocs.md` du dépôt wikiK).

## Forme

```rust
/// Compile `template` dans `work_dir`, puis copie le PDF vers `final_pdf_path`.
///
/// `work_dir` est vidé avant chaque passe : un `.aux` résiduel casse la reproductibilité.
///
/// @req EF-012
/// @adr 3
/// @pitfall xelatex absent du PATH : renvoie une erreur, ne panique jamais.
pub fn compile(/* … */) { /* … */ }
```

- **Première phrase** : le rôle, en une phrase (≈ 120 caractères). C'est ce qu'affichent `carte`, `fichier` et le wiki.
- **Ensuite** : ce que le code ne dit pas. Intention, contrat (entrées admises, erreurs, effets de bord, panics), pièges, raison d'un choix.
- **Balises** en fin de bloc, une par ligne : `@req`, `@adr`, `@topic`, `@pitfall`, `@relu`.
- **En-tête de fichier** `//!` : rôle du fichier en une phrase, puis entrées, sorties, choix de conception.

## En TypeScript

Même règles, en JSDoc : `/** … */` juste avant la déclaration (ou son `export`), décorateurs compris. En-tête de fichier : commentaires `//` en tête, ou un `/** … */` avec `@file`. Les balises JSDoc (`@param`, `@returns`…) sont permises à côté des balises wikik.

Composant Svelte : `<!-- @component … -->` en tête de fichier (convention Svelte), ou des commentaires `//` en tête du script.

```ts
/**
 * Relance la campagne en cours après une coupure réseau ; sans effet si rien ne tourne.
 *
 * @param id identifiant de campagne
 * @req EF-021
 */
export function reprendre(id: number): void { /* … */ }
```

## À éviter

- Paraphraser la signature (« Renvoie le nom de l'utilisateur » sur `fn user_name() -> String`) : `check` le signale en info.
- Décrire ligne à ligne ce que fait le corps : le bloc deviendrait faux au prochain changement.
- Écrire les dépendances à la main : elles sont calculées.
- Un bloc ADR complet dans le code : écrire l'ADR dans `docs/projet/adr/` et poser `@adr N`.

## Après une modification de code

- Le contrat ou le comportement a changé → réécrire le bloc dans le même commit.
- Le bloc reste exact → `wikik relu <id>` (ajoute `@relu AAAA-MM-JJ`).
- Puis `wikik check`.
