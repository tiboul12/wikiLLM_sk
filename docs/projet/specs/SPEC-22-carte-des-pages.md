---
id: SPEC-22
type: spec
titre: "Carte des pages : détection dans le code, corrections, complément par le LLM"
statut: proposé
version: v5
exigences: [EF-120, EF-121, EF-122]
decisions: [ADR-0010, ADR-0011]
---

# SPEC-22 : carte des pages

## Exigences

1. **R1 Sources.** Le scan lit les fichiers de l'index (même exclusions que lui) et l'index lui-même (symboles). Il ne demande aucune configuration.
2. **R2 Détecteurs**, dans cet ordre ; une page trouvée par un détecteur n'est pas reprise par les suivants (même route ou même composant) :

| Détecteur | Ce qu'il lit | Page |
|---|---|---|
| `route` | objets `{ path: '…', component: X }` ou `loadComponent: … m.X` (Angular, Vue), `<Route path="…" element={<X` (React Router), `<Route path=path!("/…") view=X` (Leptos) ; `redirectTo` ignoré | route + composant |
| `fichier-route` | `src/routes/**/+page.svelte` (SvelteKit) | route tirée des dossiers |
| `enum` | `enum Page` / `Screen` / `View` / `Ecran` / `Vue` (Rust, TS) à variantes simples ; libellé = chaîne sur la même ligne qu'un `Page::X` ; composant = première balise `<Majuscule` après `Page::X` (400 caractères au plus) | variante + libellé + composant |
| `routeur` | routeur maison à discriminant : `chemin === "/x" … return { page: "x" }` puis `route.page === "x"` suivi de `<Composant` (composant importé sous un autre nom : celui du fichier) ; la page par défaut (`accueil`, `home`) a la route `/` | route + composant |
| `html` | fichiers `.html` suivis (hors `dist`, `build`, `node_modules`) | fichier + `<title>` |
| `composant` | composants (`genre` `composant`, ou nom finissant par `Page` / `Component`) dont le fichier est sous un dossier `pages`, `views`, `routes`, `screens`, `ecrans` | composant |

3. **R3 Page.** `cle` (route, sinon nom du composant ou du fichier, en minuscules), `nom` (libellé, sinon composant découpé en mots), `route`, `symboles` (identifiants de l'index, au moins le composant trouvé par son nom), `exigences` (arêtes `req` de ces symboles), `fichier`, `source` (détecteur), `masque`, `groupe`.
4. **R4 Persistance et corrections.** Une page retrouvée sous une autre clé (sa route est maintenant connue) est reconnue à son composant et garde sa clé d'origine, à laquelle ses remarques sont rattachées. La carte est `docs/reception/carte.md` (frontmatter : `pages`). Un nouveau scan fusionne par `cle` : `nom`, `masque` et `groupe` modifiés par Tibo sont gardés (`corrige: true`), le reste est rafraîchi ; une page qui n'est plus trouvée est marquée `disparue` sans être supprimée si des remarques la citent.
5. **R5 LLM** (`completer`) : le fournisseur de l'atelier reçoit la carte actuelle et la consigne de trouver les écrans manquants en lisant le projet par MCP ; réponse `{ elements: [{ nom, route, symbole, raison }] }` ; un `symbole` inconnu de l'index est écarté ; source `llm`. Le bouton s'appelle « Chercher les écrans manquants (LLM) », une phrase dit son rôle à côté de « Scanner à nouveau » ; il se change en « Annuler la recherche » pendant l'appel ; à la fin, la liste des écrans ajoutés avec la raison du LLM (ou « rien ne manque ») (EF-146).

## Alternatives écartées

- **Liste des pages écrite à la main** : à refaire pour chaque projet (BDR-v5-1).
- **Parcourir l'application en navigateur** (crawler) : exige l'application lancée, ignore les écrans d'état (ATS), et ne relie pas au code.

## Critères d'acceptation

1. Fixture : une route Angular, une route React, une route Leptos, une page SvelteKit, un `enum Page`, une page HTML, un composant sous `pages/` : sept pages, chacune avec son composant.
2. wikik : les pages de `front/src/pages/` ; ATS (clone) : les six variantes de `Page` avec leur libellé ; shelley-studio-llm : les huit routes (sans la redirection).
3. Une page renommée et masquée le reste après un nouveau scan.
