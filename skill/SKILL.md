---
name: wikik
description: >-
  Doc dans le code et index requêtable du projet (outil wikik). À utiliser dans tout projet qui contient
  `wikik.toml`, `.wikik/` ou `docs/projet/` : AVANT d'ouvrir ou de modifier du code (interroger l'index au
  lieu de lire les fichiers), en écrivant ou corrigeant de la doc (`///`, `//!`, ADR, exigences), et avant
  chaque commit (`wikik check`). Remplace doc-as-code et wiki-dev-sync sur ces projets.
---

# wikik : interroger avant de lire, documenter en même temps que coder

`wikik` indexe la doc écrite dans le code (`///`, `//!`) et la zone Projet (`docs/projet/` : besoins, exigences, ADR, specs). L'index se met à jour tout seul à chaque requête.

## Règles d'or

1. **Interroger avant d'ouvrir.** Début de session : `wikik query carte`. Avant de toucher un symbole : `wikik query symbole <id>` puis `wikik query impact <id>`. N'ouvrir un fichier source que pour le modifier, ou si la doc ne suffit pas (`--code` donne le code d'un seul symbole).
2. **Doc et code dans le même commit.** Si tu changes le contrat, l'intention ou un piège d'une fonction, mets son bloc `///` à jour dans le même commit. Si le bloc reste juste après un changement de code : `wikik relu <id>`.
3. **Un bloc ne paraphrase pas le code.** Première phrase = rôle ; ensuite intention, contrat, pièges, raison d'un choix. Détail et exemples : `procedures/ecrire-bloc.md`.
4. **Relier.** `@req EF-xxx` quand le code implémente une exigence, `@adr N` quand il applique une décision, `@pitfall` pour un piège. Une balise par ligne, en fin de bloc.
5. **Vérifier avant de commiter.** `wikik check` : zéro erreur ; traiter chaque `perime-*` sur les symboles que tu as touchés.

## Commandes

| Besoin | Commande |
|---|---|
| Vue d'ensemble | `wikik query carte` |
| Contenu d'un fichier sans le lire | `wikik query fichier <chemin>` |
| Un symbole (doc, appels, ADR, exigences) | `wikik query symbole <id>` (`--code` pour le code) |
| Ce qu'une modification touche | `wikik query impact <id>` |
| Chercher dans la doc | `wikik query cherche <mots>` |
| Une décision / une exigence | `wikik query adr <n>` · `wikik query exigence <EF-xxx>` |
| Doc à reprendre | `wikik query perimes` |
| Référence copiée du wiki | `wikik query ref @wiki:…` |
| Vérifier | `wikik check` (`--format json`, `--strict`) |
| Marquer un bloc relu | `wikik relu <id>` |
| Créer une exigence, un besoin, un ADR, une spec | `wikik nouveau <genre> "titre"` |
| Couverture de la doc | `wikik stats` |

Identifiant de symbole : `chemin/fichier.rs::Type::methode` ; un nom partiel (`methode`, `Type::methode`) suffit s'il est unique. Chaque sortie commence par `# … · ~N tokens`.

Si le serveur MCP wikik est configuré (`claude mcp add wikik -- wikik mcp`, lancé depuis la racine du projet), utiliser ses outils `carte`, `fichier`, `symbole`, `impact`, `cherche`, `adr`, `exigence`, `perimes`, `ref`, `check` plutôt que le shell : mêmes sorties. Outils de l'atelier : `atelier`, `document`, et `atelier_rappel` quand un point du besoin est flou (voir `procedures/atelier.md`).

Si `wikik` n'est pas dans le PATH : `make install` depuis le dépôt wikiK.

## Procédures (à lire quand la situation se présente)

- `procedures/interroger.md` : trouver le bon contexte avant une modification.
- `procedures/ecrire-bloc.md` : rédiger ou corriger un bloc (bons et mauvais exemples).
- `procedures/amorcer.md` : documenter un projet existant par lots.
- `procedures/adr.md` : créer un ADR et le relier au code.
- `procedures/majeure.md` : ouvrir une mise à jour majeure (BDR puis cahier des charges).
- `procedures/atelier.md` : définir le besoin avec l'atelier, ou y rappeler un point flou pendant le développement.
