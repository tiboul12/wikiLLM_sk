---
id: BDR-v8-1
type: bdr-analyse
titre: "Analyse du besoin v8 (retours de la réception de la v7)"
statut: proposé
version: v8
sources: [BDR-v7-2, REC-042, REC-055, REC-057.2, REC-067, REC-069, REC-072, REC-078, REC-079]
---

# BDR v8, partie 1 : analyse du besoin

> Aucune solution technique dans ce document. Matière : les 41 remarques que Tibo a saisies dans le cahier de réception le 22/10/2026, en recevant la v7 (`docs/reception/remarques/REC-042` à `REC-082`).

## Contexte

Deuxième réception complète de wikik. Sept remarques sont bloquantes (Paramètres, Git, onglets), les autres gênantes. Le ton a changé : la v7 a rendu l'outil utilisable, Tibo veut maintenant **y travailler** (valider, corriger, suivre) et plus seulement y lire.

## Personnes concernées

Tibo, qui définit le besoin, relit, réceptionne et fait développer par un agent ; plus tard, l'équipe.

## Situation actuelle et irritants (avec preuves)

| Irritant | Preuve |
|---|---|
| Trop de pages ouvertes, impossible d'en garder plusieurs sous la main. | REC-069 (« j'ai trop de pages du wiki ouvertes », onglets détachables), REC-068 (un lien du graphe doit ouvrir un onglet), REC-052 (raccourcis). |
| La navigation ne range pas les outils ensemble et s'ouvre en vrac. | REC-042 (graphe et code dans un seul menu), REC-049 (accordéons ouverts à l'arrivée), REC-046 (replier sans pouvoir déplier), REC-050 (« À relire » introuvable dans l'arborescence), REC-043 (la recherche n'a pas de champ). |
| On ne voit pas ce qui attend une décision de Tibo. | REC-048 (« backlog : aucune visu de combien de tâches attendent mon approbation »), REC-051 (plans, interventions et favoris mélangés dans Gestion). |
| La relecture ne permet ni de valider ni de corriger. | REC-053 (« comment je valide ? »), REC-056 (« je ne peux rien reprendre »), REC-054 (« tout et n'importe quoi » à relire), REC-065 (pas de Ctrl+Z). |
| La documentation du projet ne descend pas en cascade jusqu'aux tests. | REC-055 (besoin → cahier des charges → exigences → un test par exigence → ticket validé par l'utilisateur ou le modèle). |
| Les paramètres sont illisibles et le LLM est imposé. | REC-057.2 (« bonne catastrophe », choisir Claude, Antigravity, Mistral ou un modèle local, le modèle et l'effort), REC-059 (« c'est quoi cette section ? »), REC-062 (connexion, puis prompts système). |
| Les consignes données aux LLM ne sont pas de vrais skills. | REC-058, REC-060 (« là c'est jeté comme ça »), REC-061 (où est le skill de documentation du code ?). |
| Le graphe est lent et ne montre pas tous les liens. | REC-045 (lourd à chaque ouverture), REC-044 (liens entre fichiers de code et entre pages), REC-070 (chercher un nœud). |
| Le Design thinking mélange ateliers, clarifications et pages de réception. | REC-072 (un atelier = une session avec titre, numéro, statut ; les clarifications à part), REC-073, REC-074, REC-075, REC-077 (pages des étapes absentes de la carte), REC-076 (bilan pré-rempli avec les besoins du projet). |
| La clarification n'est pas cohérente avec la relecture et ne juge pas la question. | REC-063 (« reprendre le modèle de relecture »), REC-064 (le hors-sujet n'est pas détecté), REC-071 (pages citées sans lien). |
| Les ateliers graphiques n'ont pas d'outil de dessin. | REC-078 (Mermaid pour la carte mentale, SVG pour le storyboard). |
| Tibo ne peut pas valider la réception ni savoir où en est une correction. | REC-079, REC-081 (valider soi-même une remarque traitée), REC-080 (fait, commité, poussé ?), REC-082 (dans quelle version). |
| L'historique du dépôt n'est pas visible. | REC-067 (« historique git à la GitKraken »). |
| Le performance de la page fichier. | REC-047. |
| Le terminal ne s'ouvre pas dans le projet. | REC-066. |

## Contraintes

- Une seule commande `wikik` (ADR-0012) ; pas d'« usine à gaz ».
- Les modifications de fichiers et l'exécution de commandes restent réservées au propriétaire (ADR-0013).
- Les données restent des fichiers du dépôt (ADR-0010).

## Hors sujet

- La réécriture du front en Rust / WebAssembly (REC-009.2) : elle passe en **v9**, toujours en attente du design system Rust de Tibo.
