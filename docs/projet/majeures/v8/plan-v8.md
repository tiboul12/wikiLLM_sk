---
id: PLAN-v8
type: plan
titre: "Plan v8 : traitement des retours de la réception de la v7"
statut: proposé
version: v8
sources: [BDR-v8-2, CDC-v8, ADR-0015, ADR-0016, ADR-0017]
---

# Plan v8 : traitement des retours de la réception de la v7

**En une phrase** : les 41 remarques du cahier de réception du 22/10/2026 sont traitées par lots, sur la branche `v8-retours` (partie de `v7-retours`, non fusionnée), puis fusionnées dans `master` après la réception de Tibo.

## Numérotation

La v8 était réservée au front en Rust / WebAssembly (REC-009.2). Cette réception passe devant : **la v8 traite les retours, le front Rust devient la v9**, toujours en attente du design system Rust de Tibo.

## Lots

| Phase | Lot | Remarques | Exigences |
|---|---|---|---|
| 1 | Coquille : outils, accordéons, recherche, terminal | REC-042, 043, 046, 049, 050, 066 | EF-150, EF-152, EF-168 |
| 2 | Onglets | REC-069, 068 | EF-151 |
| 3 | Graphe et page fichier | REC-044, 045, 070, 047 | EF-153, EF-154 |
| 4 | Paramètres : connexion et skills | REC-057.2, 058, 059, 060, 061, 062 | EF-155, EF-156 |
| 5 | Projet : à traiter, plans, favoris, relecture, édition, annulation | REC-048, 051, 052, 053, 054, 056, 065 | EF-157, EF-158, EF-159 |
| 6 | Cascade jusqu'aux tests | REC-055 | EF-160 |
| 7 | Design thinking : ateliers et clarifications | REC-063, 064, 071, 072, 073, 074, 076 | EF-161, EF-162, EF-163 |
| 8 | Réception : carte de l'atelier, validation, état git | REC-075, 077, 079, 080, 081, 082 | EF-164, EF-165 |
| 9 | Historique git | REC-067 | EF-166 |
| 10 | Dessin | REC-078 | EF-167 |

Ordre : la coquille et les onglets d'abord (tout le reste s'y range), puis ce qui est bloquant (paramètres, git), puis les écrans de travail.

## Choix par remarque (quand la remarque laisse un choix)

- **REC-042, REC-067** : barre d'activité : Accueil, Projet, Design thinking, Réception en haut ; en bas, au-dessus des Paramètres, les outils : « Code » (l'ancien Produit, avec le graphe dans son panneau) et « Git ».
- **REC-049** : accordéons fermés à l'arrivée partout sauf le panneau Code ; celui de la page ouverte s'ouvre.
- **REC-069** : onglets à la VS Code au-dessus du contenu ; « ⧉ » détache l'onglet dans une fenêtre du navigateur (même adresse, sans les onglets) ; l'onglet courant est remplacé par une navigation simple, un Ctrl+clic ou clic du milieu en ouvre un nouveau.
- **REC-043** : la page Recherche a son champ ; « Demander à l'IA » envoie la question au LLM configuré, réponse affichée avec ses liens.
- **REC-045** : graphe global gardé en mémoire du navigateur pour la session, chargé en tâche de fond après l'accueil ; « ↻ » le recharge.
- **REC-044** : arêtes fichier → fichier (appels et rendus de composants ramenés au fichier) et page → page (liens entre écrans trouvés dans le code de la page, et parent de l'arborescence).
- **REC-057.2, REC-062** : Paramètres en accordéons : 1. Connexion au LLM (ADR-0015), 2. Agent de développement, 3. Prompts système (un sous-accordéon par skill, ADR-0017), 4. Application reçue.
- **REC-059** : la section « Lecture du wiki » devient le skill `wikik-lecture`, avec une phrase qui dit quand il sert.
- **REC-061** : le skill `wikik` (documenter le code pour wikik, ex-`doc-as-code`) est livré dans le projet et visible dans les Prompts système.
- **REC-051, REC-048** : volet Projet : « À traiter » (backlog : cases non cochées des sections « À faire avec Tibo » ; documents à relire ; clarifications ouvertes), « Plans », « Favoris », puis la cascade et les autres types ; les compteurs orange remontent sur chaque accordéon.
- **REC-052** : ☆ sur l'en-tête de toute page (document, fichier, écran) ; favoris gardés dans `.wikik/favoris.json` (propres à la machine, pas versionnés).
- **REC-054** : à relire = statut `proposé` ou `brouillon` et type parmi besoin, exigence, exigence-nf, adr, spec, bdr, cahier des charges, test ; ⚙ devant chaque groupe ouvre le skill de structure de la section.
- **REC-055** : nouveau type `test` (`TEST-xxx`, `exigences: [EF-xxx]`, `validation: a_valider | utilisateur | modele | echec`) ; le volet Projet montre la cascade besoin → exigences → tests, le cahier des charges qui porte chaque exigence ; `check` : `exigence-sans-test` (info).
- **REC-072, REC-074** : panneau Design thinking : « Ateliers » (les parcours, numérotés, avec statut) et « Clarifications » (le tableau en liste, comme la relecture) ; les clarifications quittent le volet Projet.
- **REC-075, REC-077** : la route `/atelier/<étape>` déclare ses valeurs (`@ecrans` dans le code) ; la carte de réception en tire une page par étape, rangée sous le Design thinking.
- **REC-079, REC-081** : nouveau statut `validee` (par Tibo) après `traitee`, et « À reprendre » qui rouvre avec un commentaire ; disponibles dans le traitement et sur la carte de la page.
- **REC-080, REC-082** : pour chaque commit de traitement : existe, poussé (contenu dans une branche distante), première version (tag) qui le contient ; une couleur par version, légende en tête.
- **REC-078** : carte mentale : éditeur Mermaid (texte et aperçu) ; storyboard : éditeur SVG maison (rectangle, ellipse, ligne, flèche, texte, sélection, déplacement, couleur, suppression), enregistré dans l'artefact.

## Suivi

| Phase | État | Date |
|---|---|---|
| 0 BDR, CDC, exigences, ADR-0015 à 0017, plan | ✅ | 2026-10-22 |
| 1 Coquille | ✅ Code (avec le graphe) en bas au-dessus des Paramètres ; accordéons fermés à l'arrivée hors Code ; ⊟/⊞ ; « À relire » dans le volet ; recherche avec champ et « Demander à l'IA » ; terminal ouvert dans le projet (racine absolue) | 2026-10-22 |
| 2 Onglets | ✅ onglets au-dessus du contenu (fermer, glisser, gardés au rechargement), ↗ détache dans une fenêtre, Ctrl+clic et clic du milieu, le graphe ouvre en onglet | 2026-10-22 |
| 3 Graphe et page fichier | ✅ graphe global et disposition préparés en tâche de fond, gardés pour la session, ↻ ; « Trouver un nœud » ; arêtes fichier → fichier et page → page ; check en cache (2,4 s → 24 ms) | 2026-10-22 |
| 4 Paramètres | ✅ accordéons Connexion, Agent, Application, Prompts système ; connecteurs Claude Code, Antigravity, Mistral Vibe, Ollama, llama.cpp, commande libre, factice (modèles, effort, test : claude, agy et vibe essayés en vrai) ; consignes en skills `.claude/skills/` (11 skills livrés, migration des anciennes consignes) | 2026-10-22 |
| 5 Projet | ✅ Gestion (backlog, À traiter, Favoris), Plans à part, compteurs orange remontés, ⚙ skill de structure par section ; À relire limité aux décisions, Valider et Corriger ; éditeur de document ; journal d'annulation (Ctrl+Z, Ctrl+Y) | 2026-10-22 |
| 6 Cascade | ✅ type `test` (`wikik nouveau test`), page Cascade, arbre besoin → exigences → tests, tickets de validation, outil MCP `test_valider`, check `exigence-sans-test` | 2026-10-22 |
| 7 Design thinking | ✅ ateliers numérotés avec statut et métadonnées modifiables ; clarifications à part, en tableau façon relecture ; hors-sujet jugé par le skill ; pistes liées ; bilan limité au parcours | 2026-10-22 |
| 8 Réception | ✅ `@ecrans` (une page par étape, clarifications sous `/clarifications`), cahier rangé comme l'application ; validation des remarques (je valide, à reprendre) ; état git et couleur par version | 2026-10-22 |
| 9 Git | ✅ section Git : graphe des branches à couloirs, refs, situation de la branche, détail et diff | 2026-10-22 |
| 10 Dessin | ✅ éditeur Mermaid avec aperçu (carte mentale, tout diagramme) ; éditeur vectoriel SVG pour les vignettes et les maquettes d'interface | 2026-10-22 |
| Clôture | ✅ 41 remarques traitées ; TEST-001 à 020 (17 validés par le modèle, 3 à valider par Tibo : modèles locaux, hors-sujet avec un vrai LLM, terminal) ; headless clair, sombre et téléphone sans erreur ni débordement | 2026-10-22 |
