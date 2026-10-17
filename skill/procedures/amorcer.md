# Amorcer un projet existant

But : documenter progressivement, sans génération massive non relue.

1. `wikik init` (s'il n'y a ni `wikik.toml` ni `docs/projet/`), puis vérifier que `.wikik/` est ignoré par git.
2. `wikik check` : noter le nombre de `manquant` et de `manquant-entete` (point de départ).
3. Choisir **un module par lot** (un fichier ou un dossier), en commençant par le plus central : `wikik query carte`, puis le fichier dont les symboles ont le plus d'appelants (`wikik query impact`).
4. Pour ce module : écrire l'en-tête `//!` s'il manque, puis les blocs des symboles `pub` sans doc, en suivant `ecrire-bloc.md`. Lire le code de chaque symbole (`--code`) : un bloc écrit sans lire le code est une paraphrase.
5. Convertir les références en texte libre vers des balises : « voir wiki/decisions/007 » → `@adr 7` (si l'ADR existe dans `docs/projet/adr/`).
6. `wikik check`, puis proposer le lot à la relecture (un commit ou une PR par module). Ne pas enchaîner le module suivant avant la relecture.
