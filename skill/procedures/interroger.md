# Interroger avant de modifier

1. **Situer** : `wikik query carte` (une fois par session). Repérer le dossier et le fichier concernés.
2. **Lister** : `wikik query fichier <chemin>` donne l'en-tête du fichier et un résumé par symbole. `[doc périmée]` signale un bloc qui ne décrit plus le code : s'en méfier.
3. **Comprendre le symbole** : `wikik query symbole <id>` : signature, doc complète, appelants, appelés, ADR et exigences avec leur titre.
4. **Mesurer l'impact** : `wikik query impact <id>` : appelants jusqu'à 3 niveaux, décisions, exigences et pièges de l'ensemble. Une arête `(probable)` vient d'une résolution par nom : vérifier si elle compte.
5. **Lire le code seulement maintenant**, et seulement ce qu'il faut : `wikik query symbole <id> --code`, ou ouvrir le fichier pour l'éditer.
6. **Chercher si on ne connaît pas le nom** : `wikik query cherche <mots>` (blocs et documents).

Limites à garder en tête : les appels faits dans des macros (sauf composants `<Composant>` de `view!`) et les appels de méthode sur un receveur dont le nom de méthode n'est pas unique ne sont pas vus. `fichier` indique la part de code en macros.
