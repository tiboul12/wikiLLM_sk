---
id: BDR-v1-1
type: bdr-analyse
titre: Analyse du besoin v1
statut: proposé
version: v1
sources:
  - .doc/reflexion et evolution/analyse-2026-10-17.md
  - .doc/reflexion et evolution/roadmap-fusion-wikillm-docascode.md
  - docs/projet/spikes/spike-01-tree-sitter-rust.md
---

# BDR v1, partie 1 : analyse du besoin

> Aucune solution technique dans ce document.

## Contexte

Tibo développe plusieurs projets avec un agent de code (Claude Code) : ATS (Rust, 11 000 lignes), shelley-administration et shelley-studio-llm (TypeScript), et wikiK lui-même. Trois outils maison ont été essayés en septembre 2026 pour documenter ces projets : wikiK v0 (wiki Markdown écrit à la main, consultable dans le navigateur), le skill `doc-as-code` (conventions de documentation) et le skill `wiki-dev-sync` (cartographie du code régénérée en un seul fichier).

## Personnes concernées

- **Tibo**, qui pilote les projets, relit et décide.
- **L'agent de code**, qui lit le projet à chaque session et écrit du code et de la doc.
- **Plus tard, une équipe** : d'autres développeurs et leurs agents sur les mêmes dépôts.

## Situation actuelle et irritants

| Irritant | Preuve |
|---|---|
| **L'agent relit le code pour comprendre le projet.** Rien ne lui donne une vue ciblée : soit il ouvre les fichiers, soit il lit une cartographie complète. | `wiki/architecture/code-map.md` (214 lignes) est régénéré en entier et lu en entier ; `src/app.rs` d'ATS fait 2 521 lignes. Coût en tokens non mesuré à ce jour (à mesurer, voir BDR 2). |
| **La doc écrite à la main dérive du code sans que personne ne le voie.** | Le backlog v0 affichait « Historique & Intégration Git » comme terminé alors que les fonctions n'étaient branchées sur aucune route (constaté le 17/10). |
| **La doc est dispersée et dupliquée.** | Le même contenu existait dans `docs/` (Diátaxis) et `wiki/documentation/` ; les décisions vivent tantôt dans le code (`doc-as-code`), tantôt dans des fichiers ADR (`wiki/decisions/`). Les trois outils donnent des règles contradictoires. |
| **On ne sait pas ce qu'une modification va casser** avant de l'avoir faite. | Aucun outil actuel ne calcule les appelants d'une fonction. |
| **Les décisions et les exigences ne sont pas reliées au code.** | Les ADR ne pointent que vers d'autres pages ; aucune exigence n'est identifiée. |
| **Le wiki v0 n'est pas sûr à partager.** | Écriture et suppression sans authentification sur `0.0.0.0`, HTML non assaini. |

Point positif constaté : ATS est déjà documenté à 38 % (237 symboles sur 626 ont un bloc `///`), ce qui montre que l'écriture de doc dans le code est déjà une habitude.

## Contraintes

- **Un seul développeur** pour construire l'outil, en binôme avec l'agent.
- **Coût en tokens** : tout ce que l'agent lit doit être ciblé.
- **Projets en Rust d'abord**, TypeScript ensuite.
- **Rien d'hébergé** : tout tourne en local, la source de vérité reste le dépôt git.
- **Passage en équipe possible plus tard** sans tout reconstruire.

## Hors sujet

- L'atelier de conception Fast Design Thinking (idéation, storyboards) : sujet d'une majeure ultérieure.
- La recherche sémantique (embeddings).
- L'édition collaborative en temps réel.
