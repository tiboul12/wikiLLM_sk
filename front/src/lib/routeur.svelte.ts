// Routeur minimal à l'API History : URL stables (SPEC-05 R7) sans dépendance.
//
// En mode statique (site exporté), le chemin vit dans l'ancre (`#/produit/…`) : aucune configuration
// d'hébergement n'est nécessaire.
//
// @req EF-041, EF-042, EF-080

import { statique } from "./statique";
/** Page à afficher, déduite de l'URL : `produit` et `projet` sont les accueils de section, `relecture` la
 * liste des documents à relire, `clarifications` leur tableau de bord, `ateliers` l'accueil du Design thinking ; `etape` à
 * `clarifier` n'existent que dans l'atelier, `reception` à `traitement` dans le cahier de réception, et
 * `parametres` et `git`, `publication` (pas dans le site exporté).
 * @relu 2026-10-24 ba38368d
 */
export type Route =
  | { page: "accueil" }
  | { page: "produit" }
  | { page: "projet" }
  | { page: "relecture" }
  | { page: "fichier"; chemin: string; ancre: string }
  | { page: "document"; id: string; ancre: string }
  | { page: "recherche"; q: string }
  | { page: "graphe"; centre: string }
  | { page: "ateliers" }
  | { page: "atelier" }
  | { page: "etape"; etape: string }
  | { page: "clarification"; id: string }
  | { page: "clarifier"; sujet: string }
  | { page: "clarifications" }
  | { page: "reception" }
  | { page: "reception-page"; cle: string }
  | { page: "traitement" }
  | { page: "parametres" }
  | { page: "git" }
  | { page: "publication" };

/** Chemin, paramètres et ancre de l'URL courante, quel que soit le mode. */
function lireUrl(): { chemin: string; params: URLSearchParams; ancre: string } {
  if (!statique)
    return { chemin: decodeURIComponent(location.pathname), params: new URLSearchParams(location.search), ancre: decodeURIComponent(location.hash.slice(1)) };
  const h = location.hash.slice(1) || "/";
  const [avantAncre, ancre = ""] = splitUne(h, "#");
  const [chemin, requete = ""] = splitUne(avantAncre, "?");
  return { chemin: decodeURIComponent(chemin), params: new URLSearchParams(requete), ancre: decodeURIComponent(ancre) };
}

/** Coupe au premier séparateur seulement (`a/b/c` → `a`, `b/c`). */
function splitUne(t: string, sep: string): [string, string?] {
  const i = t.indexOf(sep);
  return i < 0 ? [t] : [t.slice(0, i), t.slice(i + 1)];
}

/** Route de l'URL courante ; une adresse inconnue tombe sur l'accueil.
 * @relu 2026-10-24 e0ebe39f
 */
function analyser(): Route {
  const { chemin, params, ancre } = lireUrl();
  if (chemin === "/produit" || chemin === "/produit/") return { page: "produit" };
  if (chemin === "/projet" || chemin === "/projet/") return { page: "projet" };
  if (chemin === "/projet/relecture") return { page: "relecture" };
  if (chemin.startsWith("/produit/")) return { page: "fichier", chemin: chemin.slice("/produit/".length), ancre };
  if (chemin.startsWith("/projet/")) return { page: "document", id: chemin.slice("/projet/".length), ancre };
  if (chemin === "/recherche") return { page: "recherche", q: params.get("q") ?? "" };
  if (chemin === "/graphe") return { page: "graphe", centre: params.get("centre") ?? "" };
  // `/atelier` seul est l'accueil du Design thinking (REC-091) ; `?parcours=` ouvre un atelier.
  if (chemin === "/atelier" && !params.has("parcours")) return { page: "ateliers" };
  if (chemin === "/atelier") return { page: "atelier" };
  if (chemin.startsWith("/atelier/")) return { page: "etape", etape: chemin.slice("/atelier/".length) };
  if (chemin === "/clarifications") return { page: "clarifications" };
  if (chemin.startsWith("/clarifications/clarifier/")) return { page: "clarifier", sujet: chemin.slice("/clarifications/clarifier/".length) };
  if (chemin.startsWith("/clarifications/")) return { page: "clarification", id: chemin.slice("/clarifications/".length) };
  if (chemin === "/parametres") return { page: "parametres" };
  if (chemin === "/git") return { page: "git" };
  if (chemin === "/git/publication") return { page: "publication" };
  if (chemin === "/reception") return { page: "reception" };
  if (chemin === "/reception/traitement") return { page: "traitement" };
  if (chemin.startsWith("/reception/page/")) return { page: "reception-page", cle: params.get("cle") ?? "" };
  return { page: "accueil" };
}

/** Route courante, réactive : l'application se redessine quand elle change. */
export const routeur = $state({ route: analyser() as Route });

/**
 * Navigue sans recharger la page ; ne fait rien si l'URL est déjà la bonne. En mode statique, change
 * l'ancre. `remplacer` : la page quittée ne reste pas dans l'historique (page de passage).
 */
export function aller(url: string, remplacer = false) {
  if (statique) {
    location.hash = url.replace(/^#/, "");
    return;
  }
  if (url === location.pathname + location.search + location.hash) return;
  if (remplacer) history.replaceState(null, "", url);
  else history.pushState(null, "", url);
  routeur.route = analyser();
  if (!location.hash) window.scrollTo(0, 0);
}

addEventListener("popstate", () => (routeur.route = analyser()));
addEventListener("hashchange", () => {
  if (!statique) return;
  routeur.route = analyser();
  if (!lireUrl().ancre) window.scrollTo(0, 0);
});

/** Vrai si `href` désigne la page affichée (ancre de symbole ignorée). */
export function estCourant(href: string): boolean {
  const cible = decodeURIComponent(splitUne(href.replace(/^#/, ""), "#")[0]);
  return cible === lireUrl().chemin;
}

/** Adresse de la page affichée, sous la forme que prend `aller` (ancre comprise). */
export function adresseCourante(): string {
  return statique ? location.hash || "#/" : location.pathname + location.search + location.hash;
}

/**
 * Ouverture d'un lien dans un nouvel onglet de l'application (EF-151), branchée par le module des onglets :
 * Ctrl+clic (⌘+clic) et clic du milieu sur un lien interne.
 */
export const nouvelOnglet: { ouvrir: ((href: string) => void) | null } = { ouvrir: null };

/**
 * Adresse d'un clic sur un lien interne de l'application, ou `null` (lien externe, cible ou téléchargement) pour laisser
 * faire le navigateur ; dans le site statique, seules les adresses `#/` sont internes.
 */
const lienInterne = (e: MouseEvent): string | null => {
  const a = (e.target as Element | null)?.closest?.("a");
  const href = a?.getAttribute("href");
  if (!a || !href || a.target || a.hasAttribute("download")) return null;
  if (statique ? !href.startsWith("#/") : !href.startsWith("/") || href.startsWith("//")) return null;
  return href;
};

// Les liens internes (href commençant par /) sont gérés sans rechargement ; Ctrl+clic et clic du milieu
// les ouvrent dans un nouvel onglet de l'application.
addEventListener("click", (e) => {
  if (e.defaultPrevented || e.button !== 0 || e.shiftKey || e.altKey) return;
  const href = lienInterne(e);
  if (!href) return;
  if (e.metaKey || e.ctrlKey) {
    if (!nouvelOnglet.ouvrir) return;
    e.preventDefault();
    nouvelOnglet.ouvrir(href);
    return;
  }
  if (statique) return;
  e.preventDefault();
  aller(href);
});
addEventListener("auxclick", (e) => {
  if (e.defaultPrevented || e.button !== 1 || !nouvelOnglet.ouvrir) return;
  const href = lienInterne(e);
  if (!href) return;
  e.preventDefault();
  nouvelOnglet.ouvrir(href);
});

/** Préfixe des adresses : `#` dans le site statique (pas de serveur pour répondre aux chemins), rien sinon. */
const prefixe = statique ? "#" : "";
/** Paramètre `?parcours=` d'une adresse de l'atelier ; absent pour le parcours d'origine. */
const avecParcours = (p?: string) => (p && p !== "PARCOURS" ? `?parcours=${encodeURIComponent(p)}` : "");

/** Parcours de l'atelier désigné par l'URL courante (`PARCOURS` sans paramètre). */
export function parcoursDeUrl(): string {
  return lireUrl().params.get("parcours") ?? "PARCOURS";
}

/**
 * Fabrique des URL stables : `/` (accueil), `/produit` et `/projet` (accueils de section), `/projet/relecture`,
 * `/produit/<chemin>#<symbole>`, `/projet/<id>`, `/recherche`, `/graphe`,
 * précédées de `#` en mode statique ; pour l'atelier : `/atelier` (accueil), `/atelier?parcours=<id>` (un atelier), `/atelier/<étape>` (avec
 * `?parcours=` hors du parcours par défaut), `/clarifications`, `/clarifications/<id>`, `/clarifications/clarifier/<sujet>` ; pour la réception : `/reception`, `/reception/page/?cle=<clé>`, `/reception/traitement` ; `/parametres` ; `/git` (historique), `/git/publication` (procédure de publication).
 */
export const url = {
  accueil: () => `${prefixe}/`,
  produit: () => `${prefixe}/produit`,
  projet: () => `${prefixe}/projet`,
  relecture: () => `${prefixe}/projet/relecture`,
  fichier: (chemin: string, ancre?: string) =>
    `${prefixe}/produit/${chemin.split("/").map(encodeURIComponent).join("/")}${ancre ? "#" + encodeURIComponent(ancre) : ""}`,
  document: (id: string) => `${prefixe}/projet/${encodeURIComponent(id)}`,
  recherche: (q: string) => `${prefixe}/recherche?q=${encodeURIComponent(q)}`,
  graphe: (centre?: string) => (centre ? `${prefixe}/graphe?centre=${encodeURIComponent(centre)}` : `${prefixe}/graphe`),
  // Le parcours voyage en paramètre ; le parcours par défaut n'en a pas (adresses d'avant la v7).
  ateliers: () => `${prefixe}/atelier`,
  atelier: (parcours?: string) => `${prefixe}/atelier?parcours=${encodeURIComponent(parcours || "PARCOURS")}`,
  etape: (etape: string, parcours?: string) => `${prefixe}/atelier/${encodeURIComponent(etape)}${avecParcours(parcours)}`,
  clarifications: () => `${prefixe}/clarifications`,
  // Les clarifications sont rangées sous leur tableau (REC-074).
  clarification: (id: string) => `${prefixe}/clarifications/${encodeURIComponent(id)}`,
  clarifier: (sujet: string) => `${prefixe}/clarifications/clarifier/${encodeURIComponent(sujet)}`,
  reception: () => `${prefixe}/reception`,
  // La clé d'une page est souvent une route (`/configurer`) : elle voyage en paramètre.
  pageReception: (cle: string) => `${prefixe}/reception/page/?cle=${encodeURIComponent(cle)}`,
  traitement: () => `${prefixe}/reception/traitement`,
  parametres: () => `${prefixe}/parametres`,
  // Historique git (EF-166) : un commit ou une branche à montrer voyage en paramètre.
  publication: () => `${prefixe}/git/publication`,
  git: (q?: { commit?: string; ref?: string }) =>
    `${prefixe}/git${q?.commit ? `?commit=${encodeURIComponent(q.commit)}` : q?.ref ? `?ref=${encodeURIComponent(q.ref)}` : ""}`,
};
