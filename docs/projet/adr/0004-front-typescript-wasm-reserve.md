---
id: ADR-0004
type: adr
titre: "Front en TypeScript, WebAssembly réservé au site statique"
statut: accepté
date: 2026-10-17
remplace: []
exigences: [EF-040, EF-041, EF-042, ENF-004, ENF-005]
---

# ADR-0004 : Front en TypeScript, WebAssembly réservé au site statique

## Contexte

Le wiki affiche du Markdown, des diagrammes Mermaid, un graphe, du code coloré, et plus tard des écrans d'atelier (canvas). Tibo maîtrise aussi Leptos (front Rust/WASM d'ATS).

## Décision

- Front en **TypeScript**, compilé par Vite, **embarqué dans le binaire** (`rust-embed`) : aucun CDN.
- Les types échangés avec le serveur sont **générés depuis Rust** pour éviter toute dérive.
- **WebAssembly** est réservé au moteur de requêtes compilé pour le futur site statique publié (phase équipe, backlog). *Précisé par [[adr/0008-site-statique-sans-wasm]] : le site statique de la v3 s'en passe.*
- Le choix du framework (Svelte par défaut, Leptos possible) se fait dans SPEC-05, juste avant la Phase 8, en rouvrant la question Leptos si Tibo le souhaite.

## Conséquences

- Accès direct à l'écosystème JS (Mermaid, graphe, coloration, DOMPurify).
- Deux langages dans le dépôt ; la génération des types limite le coût.

## Alternatives écartées

- **Front tout Rust/WASM (Leptos, Dioxus)** : chaque brique JS passerait par de l'interop ; à réévaluer en SPEC-05.
- **Reprendre `index.html` v0 tel quel** : dépend de CDN, HTML non assaini, écrit sur le disque.
