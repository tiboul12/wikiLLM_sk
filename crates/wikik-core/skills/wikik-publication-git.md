---
name: wikik-publication-git
description: Publie une version du projet avec git — une branche par version, des commits par lot vérifiés, la réception de l'utilisateur, puis la fusion dans la branche principale, le tag et la poussée. À utiliser quand l'utilisateur demande de commiter, pousser, fusionner, taguer ou publier une version.
---

# Publier une version avec git

## Principes

- Une version (ou une phase) = une branche, `vN-retours` ou `phase-N-nom`, partie de la branche principale ou de la version précédente si celle-ci n'est pas encore fusionnée.
- Rien n'est fusionné dans la branche principale avant que l'utilisateur ait reçu la version (remarques de réception validées, tests à valider faits).
- Pousser, fusionner, taguer et supprimer une branche sont des gestes visibles par d'autres : les annoncer et attendre l'accord de l'utilisateur, sauf consigne contraire.
- Jamais de `push --force` sur une branche partagée, jamais de réécriture de l'historique déjà poussé.

## Étape 1 : travailler sur la branche de la version

1. `git status` : partir d'un arbre propre ; ce qui traîne est signalé à l'utilisateur, pas commité d'office.
2. Créer ou reprendre la branche de la version (`git switch -c vN-retours` depuis la bonne base).
3. Un commit par lot cohérent : `make check` (ou les tests du projet), `wikik check` à zéro erreur, `wikik stats` au moins à 95 % ; message `feat:` / `fix:` / `docs:` qui cite les remarques et exigences traitées.
4. La doc dans le même commit que le code (`///`, `@req`, `@adr`, `wikik relu` si un bloc reste juste).

## Étape 2 : pousser la branche

1. Dire ce qui part : la branche, le nombre de commits (`git log --oneline origin/<branche>..`).
2. Avec l'accord de l'utilisateur : `git push -u origin <branche>`.
3. Le cahier de réception montre alors les corrections « poussées, pas encore publiées » (contour de la pastille de version).

## Étape 3 : réception par l'utilisateur

1. L'utilisateur reçoit la version (section Réception) : il valide chaque remarque traitée ou la renvoie « à reprendre ».
2. Reprendre ce qui est rouvert sur la même branche, puis repousser.

## Étape 4 : fusionner, taguer, publier

1. Quand l'utilisateur a validé la version : fusionner dans la principale en gardant la trace de la branche, `git switch master && git merge --no-ff vN-retours -m "merge: vN …"`. Si plusieurs versions s'enchaînent sans être fusionnées (v7 → v8 → v9), fusionner dans l'ordre.
2. Repasser `make check` et `wikik check` sur la principale.
3. Taguer la version (`git tag -a vN.0-rc1 -m "…"`), puis, avec l'accord de l'utilisateur, `git push origin master --follow-tags`.
4. Les pastilles du cahier de réception passent à « publiée » (pleines, couleur de la version).
5. Mettre à jour le suivi du plan de la version et le backlog ; la branche fusionnée peut être supprimée si l'utilisateur le veut.

## Garde-fous

- Un conflit de fusion se résout avec l'utilisateur quand il touche un choix (pas d'arbitrage silencieux).
- Un tag publié ne se déplace pas : une correction donne `vN.0-rc2`.
