---
id: SPEC-23
type: spec
titre: "Remarques de réception : fichiers, écrans, traitement, MCP"
statut: proposé
version: v5
exigences: [EF-123, EF-124, EF-125, EF-126, EF-127]
decisions: [ADR-0010]
---

# SPEC-23 : remarques de réception

## Exigences

1. **R1 Serveur.** Le cahier est la section « Réception » du serveur du projet (`wikik`, ADR-0012) ; écritures limitées à `docs/reception/`, mêmes gardes que l'atelier (corps JSON, même origine), corps jusqu'à 25 Mo (images). Ouvrir le cahier n'écrit rien : la carte proposée par le scan est enregistrée à la première remarque, correction ou adresse.
2. **R2 Remarque** : `docs/reception/remarques/REC-xxx.md`. Frontmatter : `id`, `type: remarque`, `titre` (début du texte), `statut` (`ouverte`, `traitee`, `reportee`, `rejetee`), `page` (clé de la carte), `nom_page`, `genre` (`bug`, `ergonomie`, `evolution`, `question`), `gravite` (`bloquant`, `genant`, `detail`), `symboles`, `exigences`, `images`, `date` ; après traitement `resolution`, `commit`. Corps : le texte, puis les images en Markdown relatif (`![capture 1](../images/REC-001-1.png)`), lisible aussi sur GitHub.
3. **R3 Images** : collées (presse-papiers) ou déposées, envoyées en `data:` URL ; PNG, JPEG, GIF, WebP seulement, 10 Mo chacune au plus ; écrites dans `docs/reception/images/`. Le serveur les sert par `/api/image?chemin=…` (même liste de types, chemin sous la racine du projet, jamais hors de `docs/`) ; le front réécrit les images relatives d'un document vers cette route.
4. **R4 Index** : `docs/reception/` est une zone de documents (onglet « Réception » du wiki) ; la carte et les remarques sont cherchables.
5. **R5 Écrans** : *Carte* (pages par groupe, compteurs de remarques ouvertes, scanner, compléter par le LLM, renommer, masquer, regrouper) ; *Page* (route, code, exigences, remarques, formulaire avec collage d'images) ; *Traitement* (filtres statut, page, gravité ; changer le statut avec une explication) ; une page « Général » pour les remarques transverses.
6. **R6 MCP** : `reception` (remarques ouvertes : identifiant, page, genre, gravité, texte, symboles, chemins des images) et `reception_traiter` (identifiant, statut, explication, commit). Seul `reception_traiter` écrit, et seulement le frontmatter et la section « Traitement » d'une remarque.

7. **R7 Application reçue** : l'adresse de l'application (`url_application` dans le frontmatter de la carte) se saisit en tête de la carte ; une adresse de boucle locale prend l'hôte du navigateur (comme les liens wiki ↔ atelier). Chaque page à route offre « ouvrir dans l'application » (adresse + route) ; la route d'une page se corrige avec ✎ ; un écran sans adresse (navigation par état, application de bureau) le dit.
8. **R8 Versions** : modifier `REC-001` écrit `REC-001.2` (puis `.3`…) avec `remplace: REC-001`, `version`, le même statut ; l'ancienne passe à `statut: remplacee` avec `remplace_par`. Les images gardées sont reprises par leur chemin, les nouvelles nommées d'après la nouvelle version. Une remarque remplacée n'apparaît ni dans les listes, ni dans l'outil `reception`, et ne se traite plus ; l'écran affiche l'historique de la chaîne.
9. **R9 Suppression** : supprimer une version retire toute sa chaîne (`REC-001`, `.2`, `.3`) et les images qu'aucune autre remarque ne cite ; l'écran demande confirmation en nommant les versions.
10. **R10 Captures** : une tuile « ＋ Capture » ouvre une aide (raccourcis de capture par système, coller avec Ctrl+V n'importe où sur la page, glisser, choisir un fichier ; « coller maintenant » quand le navigateur autorise la lecture du presse-papiers). Le collage est accepté hors du champ de texte : il va au formulaire qui a le focus, sinon à la nouvelle remarque.

## Alternatives écartées

- **Bouton « remarque » sur chaque écran de l'application reçue** : il faudrait instrumenter chaque application, et les remarques sur wikik atterriraient dans le projet ouvert.
- **Images en base64 dans le Markdown** : fichiers illisibles et lourds dans le dépôt.

## Critères d'acceptation

1. Une remarque avec deux images collées (navigateur headless) donne un fichier `REC-001.md` et deux images, affichées dans la page et dans le wiki.
2. `reception_traiter` passe une remarque à `traitee` ; elle disparaît de `reception`.
3. Une image en SVG ou un chemin hors de `docs/` est refusé.
