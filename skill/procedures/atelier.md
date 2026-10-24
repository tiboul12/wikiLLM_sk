# Atelier de définition du besoin

L'atelier (section Design thinking du serveur du projet, lancé par `wikik`) mène le framework Fast Design Thinking en 11 étapes : pitch, free writing, carte mentale, analogies, 5W1H, cubing, SCAMPER, storyboard, bilan des besoins, exigences, roadmap. Le LLM propose (`claude -p`, branché sur le serveur MCP de wikik, en lecture seule) ; Tibo garde, corrige ou rejette. Tout est écrit en fichiers : artefacts dans `docs/atelier/`, besoins, BDR, exigences et roadmap dans `docs/projet/`.

## Pour l'agent de code

1. **Un point du besoin est flou** (seuil, cas limite, deux exigences qui se contredisent) : ne pas trancher seul. Appeler l'outil MCP `atelier_rappel` avec l'identifiant concerné (`B-xxx`, `EF-xxx`, `SB-xxx`…) et une question précise. Il crée `CLAR-xxx` et renvoie son adresse : la donner à Tibo. Il garde ou corrige les pistes du LLM puis déclare la clarification finie ; sa conclusion (les pistes gardées) se lit ensuite avec l'outil `document`.
2. **Comprendre d'où vient une exigence** : outil `exigence` (énoncé, besoins), puis `document` sur ses `sources` (storyboard, idée, verbatim du free writing). La chaîne exigence → besoin → storyboard → idée → unité de sens est faite de liens réels.
3. **Lister l'atelier** : outil `atelier` (artefacts par type).
4. Ne jamais écrire dans `docs/atelier/` à la main : c'est l'atelier qui tient ces fichiers.

## Pour lancer un projet

```bash
mkdir mon-projet && cd mon-projet && git init
wikik init                 # wikik.toml, docs/projet/, gabarits
wikik                      # puis ouvrir l'adresse affichée, section Design thinking
```

`wikik` lance le serveur en arrière-plan et rend la main : il reste allumé après la fin de la connexion SSH, jusqu'au bouton ⏻ « Arrêter » en bas de la barre (ADR-0014). Relancé, `wikik` réaffiche ses adresses.

Depuis un autre poste : `reseau = true` dans la section `[serveur]` de `wikik.toml` (une fois), puis `wikik` affiche l'adresse à ouvrir. Les liens entre wiki et atelier suivent l'adresse du navigateur : aucune IP à configurer. Avec plusieurs projets ouverts, chacun prend le port suivant libre (affiché au démarrage) ; le lien renvoyé par `atelier_rappel` vise le bon.

- Mode **Express** par défaut (moins de propositions, moins de temps) ; **Complet** par étape, au besoin.
- Le pitch et le free writing sont à toi : le LLM ne fait que proposer un brouillon (pitch) et extraire les unités de sens mot pour mot (free writing).
- `[atelier] version` dans `wikik.toml` choisit la majeure documentée par le parcours par défaut (`docs/projet/majeures/<version>/`, défaut `v1`). « ＋ » dans le groupe Parcours en ouvre un autre (autre version, autre sujet), avec son propre pitch.
- Clarifications : section Projet, groupe Clarifications (rangées par sujet) ; « clarifier » sur un besoin ou une exigence ouvre directement les pistes du LLM.
- `wikik --factice` rejoue les exemples des fiches sans LLM : pour découvrir l'interface.
- Le bandeau du tableau de bord montre la consommation de l'abonnement (5 h, 7 jours) ; chaque appel est noté dans `.wikik/atelier/journal.jsonl`.

Après l'atelier, la suite est celle de `majeure.md` : faire valider BDR et cahier des charges (`statut: validé`), puis specs, ADR et code avec `@req`.
