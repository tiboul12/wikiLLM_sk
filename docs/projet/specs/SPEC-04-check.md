---
id: SPEC-04
type: spec
titre: "check : doc périmée, manquante et références cassées"
statut: proposé
version: v1
exigences: [EF-010, EF-011, EF-012, EF-013, EF-014, EF-015, ENF-002]
decisions: [ADR-0006]
---

# SPEC-04 : check

## Problème

Dire quand la doc ment : code changé sans que son bloc suive, bloc absent, référence vers une décision ou une exigence qui n'existe pas. Le faire **sans fichier d'état versionné** (conflits de merge) et **sans fausse alerte au reformatage**.

## Exigences

1. **R1** Définition exacte et testable de « périmé ».
2. **R2** Même résultat sur toute machine pour un même état du dépôt (ENF-002), donc utilisable plus tard en CI.
3. **R3** Insensible au formatage (rustfmt) et aux changements de commentaires.
4. **R4** Un humain peut déclarer un bloc relu sans le réécrire (EF-015).
5. **R5** Sortie texte et JSON, codes de retour stables (EF-014).

## Proposition

### Définition de « périmé » : référence = l'état du code quand la doc a été touchée pour la dernière fois

Pour chaque symbole **qui a un bloc** :

1. Trouver le **commit de référence** `C` : le plus récent des commits qui ont modifié une ligne du bloc de doc (`git blame` sur les lignes du bloc). Si une ligne du bloc est modifiée et non commitée, la doc est considérée comme touchée maintenant : pas de diagnostic.
2. Extraire le symbole **dans le fichier tel qu'il était au commit `C`** (même identifiant ; à défaut, même nom qualifié dans le même fichier).
3. Comparer les hash normalisés (SPEC-02) de `C` et de l'état actuel (arbre de travail) :
   - `hash_signature` différent → **`perime-signature`** ;
   - sinon `hash_corps` différent → **`perime-corps`**.
4. Symbole introuvable à `C` (déplacé, renommé) → information `perime-inconnu`.

Conséquences : un reformatage ne change pas les hash (R3) ; toucher le bloc, même pour ajouter `@relu 2026-10-17`, le rend frais (R4) ; le résultat ne dépend que du dépôt (R2).

**Hors dépôt git** (repli) : l'outil garde dans `.wikik/reference.json` le couple (`hash_doc`, `hash_corps`, `hash_signature`) vu la première fois qu'un `hash_doc` apparaît ; périmé si `hash_doc` est inchangé et que le corps ou la signature a changé. Moins fiable (perdu si `.wikik/` est effacé) : signalé en tête du rapport.

**Coût** : un `git blame --porcelain` par fichier qui contient des blocs, et un `git show C:chemin` par couple (commit, fichier) distinct ; résultats mis en cache dans `.wikik/cache-git/` par (commit, chemin), donc invariants.

### Diagnostics

| Code | Sévérité par défaut | Condition |
|---|---|---|
| `perime-signature` | avertissement | voir plus haut |
| `perime-corps` | avertissement | voir plus haut |
| `perime-inconnu` | info | symbole introuvable au commit de référence |
| `manquant` | avertissement | symbole au-dessus du seuil de visibilité (SPEC-01 R5), hors test, sans bloc |
| `manquant-entete` | info | fichier de code sans `//!` |
| `ref-adr` | erreur | `@adr` vers un ADR absent de la zone Projet |
| `ref-req` | erreur | `@req` vers une exigence absente |
| `lien-casse` | erreur | lien `[[...]]` ou champ de frontmatter vers un document absent |
| `frontmatter-invalide` | erreur | YAML illisible |
| `exigence-non-couverte` | avertissement | exigence `must`, statut `validé` ou `implémenté`, sans aucune arête `req` entrante |
| `balise-inconnue` | avertissement | balise hors liste de SPEC-01 |
| `parse-partiel` | avertissement | `erreurs_parse > 0` |
| `paraphrase`, `bloc-long` | info | heuristiques de SPEC-01 |

Sévérités réglables dans `wikik.toml` :

```toml
[check]
seuil_visibilite = "pub"            # pub | pub-crate | tout
[check.severites]
perime-corps = "erreur"             # erreur | avertissement | info | ignore
```

### Sorties

- **Texte** (défaut) : groupé par fichier, trié par ligne ; résumé final `N erreurs, N avertissements, N infos`.
- **JSON** (`--format json`) :

```json
{ "version": 1,
  "mode_reference": "git",
  "diagnostics": [ { "code": "perime-corps", "severite": "avertissement",
                     "fichier": "src/photo.rs", "ligne": 56,
                     "symbole": "src/photo.rs::set_from_file",
                     "message": "le corps a changé depuis la dernière modification du bloc (commit 1a2b3c4)" } ],
  "resume": { "erreurs": 0, "avertissements": 1, "infos": 0 } }
```

Tri : fichier, ligne, code, symbole.
- **Codes de retour** : `0` sans erreur ; `1` au moins une erreur (ou au moins un avertissement avec `--strict`) ; `2` échec de l'outil.

### `wikik relu <symbole>`

Ajoute ou met à jour la ligne `/// @relu AAAA-MM-JJ <jeton>` en fin du bloc du symbole, où `<jeton>` est l'empreinte (8 caractères) de la signature et du corps relus. C'est la seule commande v1 qui écrit dans le code ; elle n'écrit que cette ligne.

Un symbole dont le jeton `@relu` correspond à son code actuel n'est jamais signalé périmé, quel que soit l'historique git. Ajouté après usage : sans jeton, relancer `relu` le même jour sur un bloc déjà marqué ne changeait pas le texte, et le bloc restait périmé.

## Alternatives écartées

- **Fichier d'état versionné (`doc.lock`)** : conflits de merge à chaque PR.
- **Comparer les dates de dernier commit du corps et du bloc** : un `cargo fmt` sur le corps produirait une fausse alerte ; la comparaison de hash normalisés l'évite.
- **Diff de PR seulement** : ne dit rien hors PR, alors que l'usage v1 est local.

## Questions ouvertes

Aucune bloquante.

## Critères d'acceptation

Sur un dépôt git de test créé par le test lui-même :
1. Commit 1 : fonction documentée. Commit 2 : corps modifié, bloc intact → `perime-corps`.
2. Commit 3 : ajout de `@relu` → plus de diagnostic.
3. Commit 4 : `rustfmt` agressif → aucun diagnostic.
4. Corps modifié non commité, bloc intact → `perime-corps` ; bloc modifié non commité → rien.
5. `@req EF-999` → `ref-req` (erreur), code de retour 1.
6. Sortie JSON identique sur deux exécutions.
