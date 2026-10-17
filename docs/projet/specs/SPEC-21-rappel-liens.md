---
id: SPEC-21
type: spec
titre: "Atelier : liens avec le wiki et rappel en développement"
statut: proposé
version: v4
exigences: [EF-106, EF-109, EF-110]
decisions: [ADR-0009]
---

# SPEC-21 : liens et rappel

## Exigences

1. **R1** Le wiki lit `docs/atelier/` (onglet « Atelier » à côté de Produit et Projet) ; une page de besoin, d'exigence ou d'artefact affiche « ouvrir dans l'atelier » si l'URL de l'atelier est connue (`wikik.toml`, `[atelier] url`, défaut `http://127.0.0.1:8485`).
2. **R2** L'atelier affiche « voir dans le wiki » (`[atelier] url_wiki`, défaut `http://127.0.0.1:8484`).
   Mise en œuvre : l'atelier sert aussi l'API du wiki, donc ses liens vers les documents restent dans l'atelier ; le tableau de bord donne en plus l'adresse de `wikik serve`. Côté wiki, « ouvrir dans l'atelier » mène à l'étape qui a produit l'artefact (`technique`) et « clarifier » à `/clarifier/<id>`, où Tibo pose sa question. Le front lit `/api/mode` pour savoir quel serveur le sert ; le site exporté n'a ni ces liens ni cette route.
3. **R3** Outils MCP ajoutés : `atelier` (liste des artefacts par type : identifiants et titres), `document` (contenu d'un document de l'atelier ou du projet par identifiant), `atelier_rappel` (crée une clarification `CLAR-xxx` sur un identifiant avec une question, renvoie son URL dans l'atelier). Seul `atelier_rappel` écrit, et seulement un fichier `CLAR-`.
4. **R4** L'écran de clarification charge le sujet et ses sources ; le LLM répond en lisant le projet par MCP ; Tibo conclut ; la conclusion est écrite dans `CLAR-xxx` (`etat: close`).
5. **R5** La page du sujet dans le wiki liste ses clarifications (arête `sujet`).

## Critères d'acceptation

Sur une copie d'ATS : `atelier_rappel AC-019 "…"` crée `docs/atelier/clarifications/CLAR-001.md` ; après conclusion, la page de `AC-019` dans le wiki la liste.
