// Routeur minimal à l'API History : URL stables (SPEC-05 R7) sans dépendance.
//
// En mode statique (site exporté), le chemin vit dans l'ancre (`#/produit/…`) : aucune configuration
// d'hébergement n'est nécessaire.
//
// @req EF-041, EF-042, EF-080

import { statique } from "./statique";
/** Page à afficher, déduite de l'URL ; les trois dernières n'existent que dans l'atelier. */
export type Route =
  | { page: "accueil" }
  | { page: "fichier"; chemin: string; ancre: string }
  | { page: "document"; id: string; ancre: string }
  | { page: "recherche"; q: string }
  | { page: "graphe"; centre: string }
  | { page: "atelier" }
  | { page: "etape"; etape: string }
  | { page: "clarification"; id: string };

/** Chemin, paramètres et ancre de l'URL courante, quel que soit le mode. */
function lireUrl(): { chemin: string; params: URLSearchParams; ancre: string } {
  if (!statique)
    return { chemin: decodeURIComponent(location.pathname), params: new URLSearchParams(location.search), ancre: decodeURIComponent(location.hash.slice(1)) };
  const h = location.hash.slice(1) || "/";
  const [avantAncre, ancre = ""] = splitUne(h, "#");
  const [chemin, requete = ""] = splitUne(avantAncre, "?");
  return { chemin: decodeURIComponent(chemin), params: new URLSearchParams(requete), ancre: decodeURIComponent(ancre) };
}

function splitUne(t: string, sep: string): [string, string?] {
  const i = t.indexOf(sep);
  return i < 0 ? [t] : [t.slice(0, i), t.slice(i + 1)];
}

function analyser(): Route {
  const { chemin, params, ancre } = lireUrl();
  if (chemin.startsWith("/produit/")) return { page: "fichier", chemin: chemin.slice("/produit/".length), ancre };
  if (chemin.startsWith("/projet/")) return { page: "document", id: chemin.slice("/projet/".length), ancre };
  if (chemin === "/recherche") return { page: "recherche", q: params.get("q") ?? "" };
  if (chemin === "/graphe") return { page: "graphe", centre: params.get("centre") ?? "" };
  if (chemin === "/atelier") return { page: "atelier" };
  if (chemin.startsWith("/atelier/")) return { page: "etape", etape: chemin.slice("/atelier/".length) };
  if (chemin.startsWith("/clarification/")) return { page: "clarification", id: chemin.slice("/clarification/".length) };
  return { page: "accueil" };
}

/** Route courante, réactive : l'application se redessine quand elle change. */
export const routeur = $state({ route: analyser() as Route });

/** Navigue sans recharger la page ; ne fait rien si l'URL est déjà la bonne. En mode statique, change l'ancre. */
export function aller(url: string) {
  if (statique) {
    location.hash = url.replace(/^#/, "");
    return;
  }
  if (url === location.pathname + location.search + location.hash) return;
  history.pushState(null, "", url);
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

// Les liens internes (href commençant par /) sont gérés sans rechargement.
addEventListener("click", (e) => {
  if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
  const a = (e.target as Element | null)?.closest?.("a");
  const href = a?.getAttribute("href");
  if (!a || !href || !href.startsWith("/") || href.startsWith("//") || a.target) return;
  e.preventDefault();
  aller(href);
});

const prefixe = statique ? "#" : "";

/**
 * Fabrique des URL stables : `/produit/<chemin>#<symbole>`, `/projet/<id>`, `/recherche`, `/graphe`,
 * précédées de `#` en mode statique ; pour l'atelier : `/atelier`, `/atelier/<étape>`, `/clarification/<id>`.
 */
export const url = {
  accueil: () => `${prefixe}/`,
  fichier: (chemin: string, ancre?: string) =>
    `${prefixe}/produit/${chemin.split("/").map(encodeURIComponent).join("/")}${ancre ? "#" + encodeURIComponent(ancre) : ""}`,
  document: (id: string) => `${prefixe}/projet/${encodeURIComponent(id)}`,
  recherche: (q: string) => `${prefixe}/recherche?q=${encodeURIComponent(q)}`,
  graphe: (centre?: string) => (centre ? `${prefixe}/graphe?centre=${encodeURIComponent(centre)}` : `${prefixe}/graphe`),
  atelier: () => `${prefixe}/atelier`,
  etape: (etape: string) => `${prefixe}/atelier/${encodeURIComponent(etape)}`,
  clarification: (id: string) => `${prefixe}/clarification/${encodeURIComponent(id)}`,
};
