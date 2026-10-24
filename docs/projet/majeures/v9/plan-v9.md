---
id: PLAN-v9
type: plan
titre: "Plan v9 : traitement des retours de la réception de la v8"
statut: proposé
version: v9
sources: [BDR-v9-2, CDC-v9]
---

# Plan v9 : traitement des retours de la réception de la v8

**En une phrase** : les 38 remarques ouvertes du cahier de réception au 24/10/2026 sont traitées par lots, sur la branche `v9-retours` (partie de `v8-retours`, non fusionnée), puis fusionnées dans `master` après la réception de Tibo.

## Numérotation

La v9 était réservée au front en Rust / WebAssembly (REC-009.2). Cette réception passe devant : **la v9 traite les retours, le front Rust et le design system deviennent la v10** (décision de Tibo, 24/10/2026).

## Lots

| Phase | Lot | Remarques | Exigences |
|---|---|---|---|
| 1 | Corrections rapides | REC-084, 103, 112, 109, 088, 089, 100 | EF-182, EF-170, EF-180, EF-183, EF-169, EF-171 |
| 2 | Onglets et cascades | REC-069, 102 | EF-169, EF-170 |
| 3 | Paramètres et prompts de page | REC-062, 104.3, 105, 106, 107 | EF-176 |
| 4 | Projet par version | REC-101.2, 090, 083, 099, 114 | EF-174, EF-175, EF-171, EF-180 |
| 5 | Réception | REC-082, 085, 086, 092 | EF-181 |
| 6 | Graphe | REC-002, 003 | EF-184 |
| 7 | Design thinking | REC-091, 098, 094, 093, 095, 096, 113, 115 | EF-177, EF-178, EF-179, EF-180, EF-172 |
| 8 | Git | REC-108, 110, 111 | EF-183 |
| 9 | Recherche | REC-087, 097 | EF-172, EF-173 |

Ordre : les corrections rapides d'abord (risque nul), puis la coquille (onglets), puis les bloquants (paramètres, projet, atelier), la recherche sémantique en dernier (la plus incertaine).

## Choix par remarque (quand la remarque laisse un choix)

- **REC-069** : onglet d'aperçu à la VS Code : un clic remplace l'aperçu (titre en italique), double-clic sur l'onglet, modification, Ctrl+clic ou clic du milieu le fixe.
- **REC-084** : la page « X » venait de l'exemple `'/x'` d'un bloc de doc du scan des routes ; le scan ignore les commentaires.
- **REC-101.2** : arbre par version (ADR-0018) : demande d'évolution, BDR analyse, BDR définition, cahier des charges → besoins → exigences → tests ; ADR, specs, plan de la version à côté ; ce qui n'a pas de version reste dans « Transverse ».
- **REC-114** : « il ne faut plus que clarifier » : sur une page du Projet, « clarifier » est le seul renvoi vers le Design thinking (« ouvrir dans le Design thinking » ne reste que sur les artefacts de l'atelier).
- **REC-082** : la version d'une correction est celle du tag qui la publie, sinon celle de la branche `vN-…` où elle est faite (v7 à v9 n'ont pas encore de tag : tout paraissait « poussé, pas publié ») ; pastille « v8 » dans la teinte de sa version, pleine si publiée, en contour si poussée, en tirets si seulement commitée.
- **REC-110** : l'historique paraissait linéaire parce que `v7-retours` et `v8-retours` ne sont pas fusionnées ; le dessin des branches le montre.
- **REC-097** : ADR-0020 avant le code.
- **REC-098, REC-094** : ADR-0019 : fiche corrigée par atelier dans `<dossier>/.techniques/`, mode choisi pour tout l'atelier.

## Suivi

| Phase | État | Date |
|---|---|---|
| 0 BDR, CDC, exigences, plan | ✅ | 2026-10-24 |
| 1 Corrections rapides | ✅ scan des routes sans les commentaires (plus de page « X ») ; page Cascade retirée ; « Pistes » ; branches sans puce, locales et distantes ouvertes pareil ; onglets plus hauts sans barre de défilement visible (molette) ; fermer l'onglet actif affiche son voisin ; icônes seules dans la barre du téléphone | 2026-10-24 |
| 2 Onglets et cascades | ✅ onglet d'aperçu en italique remplacé à chaque navigation, fixé par double-clic, écriture, Ctrl+clic ou clic du milieu ; une page déjà ouverte réactive son onglet ; arborescences du volet repliables nœud par nœud | 2026-10-24 |
| 3 Paramètres et prompts de page | ✅ deux blocs : Connexion (LLM de wikik, agent du terminal, réglages avancés repliés) et Prompts système généraux ; modèle en vraie liste (avec « autre »), aligné sur l'effort ; « Application reçue » retirée ; encart ⚙ du skill sur la page qui s'en sert (documents, clarification, réception) ; ⚙ du volet retirés | 2026-10-24 |
| 4 Projet par version | ✅ ADR-0018 ; volet et accueil rangés par version (demande d'évolution = sources de la BDR analyse, analyse, définition, cahier des charges → besoins → exigences → tests, ENF, ADR, specs, plan, autres), transverses à part ; état de suivi par version à l'accueil ; « à traiter » sur la boîte Projet de l'accueil ; « clarifier » seul renvoi des pages Projet | 2026-10-24 |
| 5 Réception | ✅ pastille de version (tag, sinon branche) partout, légende ; un seul composant de remarque, avec curseur ouverte → traitée → validée, reporter et rejeter à côté ; tuile capture du téléphone : presse-papiers direct en https, sinon zone où coller par appui long et raccourcis du téléphone | 2026-10-24 |
| 6 Graphe | ✅ zoom à la molette ×1,6 au premier cran puis de plus en plus fin dans le même geste, jusqu'à ×12 ; double-clic : zoom sur le nœud (le clic simple attend 260 ms) ; appui long : le reste gris et sans titre, les nœuds liés cerclés de leur couleur, listés sous le graphe | 2026-10-24 |
| 7 Design thinking | ✅ ADR-0019 ; accueil du Design thinking (cartes des ateliers, création, clarifications), un atelier sur `/atelier?parcours=` ; mode express ou complet pour tout l'atelier, techniques cochées en express ; ✎ Modifier l'atelier aussi hors du poste du propriétaire ; fiche de chaque technique corrigible pour l'atelier (`.techniques/`) ; axe en pause, proposition de l'axe au-dessus de l'écriture ; clarification : recherche dynamique floue du sujet, fiche du sujet réaffichée et envoyée en entier au LLM | 2026-10-24 |
| 8 Git | ✅ fichiers non commités dépliables, groupés par dossier, en liens ; tableau des branches par rapport à la principale (partie de, avance, retard, fusionnée) et pourquoi elles se suivent en ligne ; « jamais poussée » ; skill `wikik-publication-git` et page Git › Publication (étapes cochées d'après le dépôt, « Publier avec l'agent ») | 2026-10-24 |
