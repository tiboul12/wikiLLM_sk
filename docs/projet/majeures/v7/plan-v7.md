---
id: PLAN-v7
type: plan
titre: "Plan v7 : traitement des retours de réception"
statut: validé
version: v7
sources: [BDR-v7-2, CDC-v7, ADR-0013]
---

# Plan v7 : traitement des retours de réception

**En une phrase** : les 40 remarques du cahier de réception du 19/10/2026 sont traitées par lots, sur une branche temporaire `v7-retours`, puis fusionnées dans `master`.

## Décision : la réécriture en Rust / WebAssembly passe en v8

REC-009.2 demande de refaire tout le front avec `leptos-shadcn-ui`. **Recommandation retenue : d'abord les 39 autres remarques (v7), puis la réécriture (v8).**

- Les remarques changent surtout ce que montrent les écrans (données, API, cœur) : ce travail est gardé par la réécriture.
- Réécrire un front stabilisé est un portage ; réécrire un front qu'on corrige en même temps mélange deux risques.
- La v8 commence par sa BDR et un ADR qui remplace ADR-0004 (front TypeScript) ; la réception de la v7 sert de référence d'écrans.

## Lots

| Phase | Lot | Remarques | Exigence |
|---|---|---|---|
| 1 | Noms et navigation | REC-014, 015, 016, 027, 028, 029, 036, 037, 008, 010, 011, 026 | EF-140 |
| 2 | Graphe | REC-001.2, 002, 003, 004, 005, 006 | EF-141 |
| 3 | Produit et fichiers | REC-007, 020, 021, 023 | EF-142 |
| 4 | Projet : gestion, à relire, liens | REC-012, 013, 017.2, 018.2 | EF-142, EF-143 |
| 5 | Clarifications et parcours | REC-024.2, 025, 032, 033, 034, 035, 040.2, 039 | EF-144, EF-145 |
| 6 | Complétion de la carte | REC-030, 031 | EF-146 |
| 7 | Paramètres, terminal, envoi du traitement | REC-019, 022, 041 | EF-147, EF-148 |
| v8 | Front en Rust / WebAssembly | REC-009.2 | à écrire |

En fin de phase : `make check`, contrôle headless, puis chaque remarque du lot passe à « traitée » avec sa résolution et son commit (`reception_traiter`).

## Choix par remarque (quand la remarque laisse un choix)

- **REC-015** : ordre de la barre : Accueil, Projet, Design thinking, Produit, Réception ; Paramètres en bas.
- **REC-027, REC-028** : `/reception` s'appelle « Cahier de réception » ; l'écran d'une page s'appelle « Carte de réception » (une carte par page, qui porte ses remarques).
- **REC-026** : arborescence tirée des routes (préfixe le plus long), les écrans sans route rangés sous l'écran de leur dossier ; on la corrige avec ✎ (route, groupe). Un écran disparu du code dont les remarques sont traitées ne s'affiche plus (case « afficher les masquées »).
- **REC-029** : le champ d'adresse disparaît du cahier ; l'adresse est déduite, ou réglée dans les paramètres.
- **REC-001.2** : catégories : code, exigences et besoins, décisions, documents de projet, atelier (avant-projet), pages de l'application, remarques ; une couleur et une puce activable chacune.
- **REC-006** : disposition déterministe (même graphe, même dessin).
- **REC-017.2** : groupe « Gestion » en tête du volet Projet (backlog, plans) ; un plan s'affiche en colonnes « à faire / en cours / fait » à partir de son tableau de suivi.
- **REC-018.2** : tout identifiant connu, chemin de fichier ou symbole cité dans un document devient un lien.
- **REC-024.2, REC-025, REC-040.2** : un seul écran de clarification par sujet : il s'ouvre sur les pistes du LLM, sans question ni conclusion à écrire ; le sujet est « clarifié » quand Tibo le déclare fini. Les clarifications forment un groupe à part dans le volet Projet, rangées par sujet.
- **REC-039** : plusieurs parcours, chacun dans son fichier ; « + » dans le groupe Parcours en crée un.
- **REC-022, REC-041** : volet droit avec un terminal ouvert dans le projet (où Tibo lance son agent) ; « Traiter les remarques » y tape la consigne de traitement, réglable dans les paramètres et qui impose de commencer par un plan.

## Suivi

| Phase | État | Date |
|---|---|---|
| 0 BDR, CDC, plan, ADR-0013 | ✅ | 2026-10-19 |
| 1 Noms et navigation | ✅ barre Accueil, Projet, Design thinking, Produit, Réception ; titre du panneau → accueil de section ; « replier tout » ; panneau à largeur réglable ; élément ouvert déplié et centré ; arborescence des pages ; « Cahier » et « Carte de réception » ; « En bref » des étapes | 2026-10-19 |
| 2 Graphe | ✅ graphe global : documents, code ramené au fichier, pages, remarques ; 7 catégories colorées en puces activables ; disposition à graine fixe ; zoom ×5, espacement ; titre au survol ; appui long : liens colorés (Échap pour effacer) | 2026-10-19 |
| 3 Produit et fichiers | ✅ tableau fichier par fichier (lignes, symboles, documentés, sans doc, périmés, alertes ; tri, filtre) ; récapitulatif de conformité en tête de fichier ; nom du symbole = lien direct ; ⚓ copie aussi hors HTTPS ; liens d'appels repliés dans le cadre | 2026-10-19 |
| 4 Projet : gestion, à relire, liens | ✅ groupe Gestion en tête (backlog, plans) ; plans en colonnes à faire / en cours / fait, backlog en colonnes par section ; page « À relire » (92 documents, statut, type, version, filtres) ; identifiants, chemins et `chemin::Symbole` cités liés d'office dans les documents et la doc du code | 2026-10-19 |
| 5 Clarifications et parcours | ✅ clarifications dans Projet, rangées par sujet avec leur état ; tableau de bord ; « clarifier » ouvre directement les pistes (question par défaut, clarification ouverte reprise) ; pistes gardées = réponse, « Clarifié : terminer », rouvrir, supprimer ; « Reprendre » depuis la page du document ; plusieurs parcours (`PARCOURS-n`, pitch et carte à part, version propre), « + » dans le volet | 2026-10-19 |
| 6 Complétion de la carte | ✅ « Chercher les écrans manquants (LLM) » expliqué, « Annuler la recherche » et « Annuler » dans le bandeau (atelier compris) ; le CLI et ses sous-processus sont arrêtés ; écrans ajoutés listés avec la raison du LLM | 2026-10-19 |
| 7 Paramètres, terminal, envoi du traitement | ✅ section Paramètres (agent, consigne de traitement, méthode de lecture, LLM, adresse) écrite dans wikik.toml ; terminal intégré (PTY + WebSocket, xterm.js) dans un volet droit ; « Traiter avec l'agent » ; accès réservé à la machine locale ou à la clé du lancement (SPEC-24, ADR-0013) | 2026-10-19 |
