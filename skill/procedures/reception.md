# Cahier de réception

La section Réception du serveur du projet (`wikik`) est le cahier de réception : la carte de ses pages, construite à partir du code (routes Angular / Vue / React / Leptos, SvelteKit, `enum Page`, pages HTML, composants sous `pages/`), et les remarques de l'utilisateur, une carte par remarque, captures comprises. Tout est écrit dans `docs/reception/` : `carte.md`, `remarques/REC-xxx.md`, `images/`.

## Pour l'utilisateur

1. `wikik` dans le projet à recevoir (avec `[serveur] reseau = true` pour l'ouvrir depuis un autre poste), puis l'adresse affichée, section Réception.
2. La carte est proposée à partir du code ; elle s'enregistre à la première remarque ou correction. La corriger si besoin : ✎ pour renommer, regrouper ou masquer ; « Compléter avec le LLM » cherche les écrans que le scan n'a pas vus (vues par état, étapes, fenêtres). Les corrections survivent aux scans suivants.
3. Saisir une fois, en tête de la carte, l'adresse de l'application reçue (`http://10.0.0.21:4200`) : chaque page a alors son lien « ouvrir dans l'application » (une page sans route : ✎ pour l'ajouter).
4. Ouvrir une page, écrire la remarque, ajouter les captures (« ＋ Capture » rappelle les raccourcis ; Ctrl+V marche n'importe où sur la page, glisser aussi), choisir genre et gravité, « Ajouter la remarque » (Ctrl+Entrée). Une remarque transverse va sur « Général ».
5. Corriger une remarque : « Modifier » écrit une nouvelle version (`REC-001.2`), l'ancienne reste dans l'historique. « Supprimer » retire toute la chaîne, après confirmation ; un numéro n'est jamais réutilisé.
6. En fin de campagne : « Traiter les remarques », ou laisser l'agent de code le faire.

## Pour l'agent de code

1. Outil MCP `reception` (ou `wikik query reception`) : les remarques ouvertes, les bloquantes d'abord, avec le code de la page (`chemin::Symbole`), ses exigences et les chemins des captures. Lire les captures avec Read.
2. Partir du symbole donné (`symbole`, `impact`) : pas de recherche à l'aveugle.
3. Corriger, commiter, puis `reception_traiter` : `traitee` avec une phrase et le hash du commit ; `reportee` (va au backlog) ou `rejetee` avec la raison. Une remarque qui demande une décision produit : la laisser ouverte et la signaler à l'utilisateur.
4. Ne jamais modifier `docs/reception/` à la main : c'est le cahier qui tient ces fichiers.
