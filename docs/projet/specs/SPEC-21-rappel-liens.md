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

1. **R1** Le wiki lit `docs/atelier/` (section « Atelier ») ; une page de besoin, d'exigence ou d'artefact affiche « ouvrir dans l'atelier » et « clarifier » (liens internes : un seul serveur, ADR-0012).
2. **R2** L'atelier et le wiki sont deux sections du même serveur : les documents s'ouvrent sur place. L'adresse renvoyée par `atelier_rappel` est celle du serveur du projet (port réellement pris, IP du réseau avec `[serveur] reseau = true`).
3. **R3** Outils MCP ajoutés : `atelier` (liste des artefacts par type : identifiants et titres), `document` (contenu d'un document de l'atelier ou du projet par identifiant), `atelier_rappel` (crée une clarification `CLAR-xxx` sur un identifiant avec une question, renvoie son URL dans l'atelier). Seul `atelier_rappel` écrit, et seulement un fichier `CLAR-`.
4. **R4** L'écran de clarification charge le sujet et ses sources et demande d'office des pistes au LLM, qui lit le projet par MCP ; Tibo garde, corrige ou rejette : les pistes gardées font la réponse, écrite en conclusion de `CLAR-xxx` quand il déclare la clarification finie (`etat: close`). « Clarifier » sur un document ne demande rien à écrire (EF-144).
5. **R5** La page du sujet dans le wiki liste ses clarifications (arête `sujet`).

## Critères d'acceptation

Sur une copie d'ATS : `atelier_rappel AC-019 "…"` crée `docs/atelier/clarifications/CLAR-001.md` ; après conclusion, la page de `AC-019` dans le wiki la liste.
