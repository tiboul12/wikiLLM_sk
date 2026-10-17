---
id: SPEC-16
type: spec
titre: "Équipe : export statique, CI, annotations, identifiants, statistiques"
statut: proposé
version: v3
exigences: [EF-080, EF-081, EF-082, EF-083, EF-084, EF-085, ENF-002, ENF-004]
decisions: [ADR-0008, ADR-0003]
---

# SPEC-16 : Équipe

## Exigences

1. **R1 Export** (`wikik export <dossier>`) : le dossier contient le front (index.html réécrit en mode statique, assets en chemins relatifs) et `donnees/` :

| Fichier | Contenu |
|---|---|
| `donnees/projet.json` | `wiki::projet` |
| `donnees/check.json` | rapport de `check` |
| `donnees/graphe.json` | graphe des documents |
| `donnees/graphe-complet.json` | tous les nœuds et arêtes (graphes locaux calculés dans le navigateur) |
| `donnees/recherche.json` | `cle`, `genre`, `titre`, `texte` (2 000 caractères au plus), `fichier` |
| `donnees/fichiers/<chemin>.json` | `wiki::fichier` |
| `donnees/documents/<id>.json` | `wiki::document` |

   Noms de fichiers : chaque caractère hors `[A-Za-z0-9._-]` (et hors `/` pour les chemins) devient `~` suivi de son code hexadécimal UTF-8 (`EF-001` reste `EF-001`, `a b` devient `a~20b`). Même fonction en Rust et en TypeScript. L'export est déterministe (ENF-002) : relancé sur le même commit, il produit les mêmes octets.
2. **R2 Mode statique du front** : détecté par `<meta name="wikik-statique">` ; routage par ancre (`#/produit/…`) ; lecture des fichiers de `donnees/` ; recherche : tous les mots (sans casse ni accents) présents dans le titre ou le texte, titres d'abord.
3. **R3 Annotations** (`check --format github`) : une ligne `::error|warning|notice file=…,line=…,title=<code>::<message>` par diagnostic ; `%`, `\r`, `\n` échappés (et `:` `,` dans les propriétés) ; code de retour inchangé.
4. **R4 CI** (`wikik init --ci`) : `.github/workflows/wikik.yml` (check annoté sur PR et push ; export publié sur GitHub Pages depuis la branche principale) ; `.github/CODEOWNERS` si `[equipe] validateurs` est renseigné dans `wikik.toml` (règle sur `docs/projet/majeures/` et `docs/projet/exigences/`). Rien n'est réécrit s'il existe.
5. **R5 Hook** (`wikik init --hook`) : `pre-commit` dans le dossier de hooks de git, qui lance `wikik check` ; refus si un hook existe déjà (message explicatif).
6. **R6 Identifiants** (`wikik nouveau <genre> "titre"`) : genres `exigence` (`EF-`, 3 chiffres), `exigence-nf` (`ENF-`), `besoin` (`B-`), `adr` (`ADR-`, 4 chiffres, fichier `adr/NNNN-titre.md`), `spec` (`SPEC-`, 2 chiffres, `specs/SPEC-NN-titre.md`). Numéro = plus grand numéro existant (index et fichiers) + 1. Gabarit embarqué, frontmatter rempli (`id`, `titre`, `statut: proposé`, `date` pour un ADR). Une collision entre deux branches reste possible : `check` la signale (`id-duplique`, erreur) et la CI la bloque.
7. **R7 Statistiques** (`wikik stats`) : fichiers, symboles (hors tests et champs), documentés, taux, périmés, exigences `must` et `must` couvertes ; texte ou JSON.

## Alternatives écartées

- **Routage par chemin sur le site statique** : exigerait une réécriture côté serveur ou l'astuce `404.html` de GitHub Pages, fragile sous un sous-chemin.
- **Numérotation centralisée des identifiants** (service, fichier verrou) : contraire à A1 ; la détection de doublon en CI suffit.

## Critères d'acceptation

1. L'export de wikiK, servi par un serveur de fichiers quelconque, affiche accueil, fichier, document, recherche et graphe dans un navigateur headless, sans erreur de console.
2. Deux exports successifs sont identiques (`diff -r`).
3. `check --format github` sur la fixture produit une ligne `::error` pour `ref-req`.
4. `wikik nouveau exigence` sur la fixture crée `EF-003.md`.
