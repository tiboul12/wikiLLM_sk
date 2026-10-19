// Routeur minimal à l'API History : URL stables (SPEC-05 R7) sans dépendance.
//
// En mode statique (site exporté), le chemin vit dans l'ancre (`#/produit/…`) : aucune configuration
// d'hébergement n'est nécessaire.
//
// @req EF-041, EF-042, EF-080

import { statique } from "./statique";
/** Page à afficher, déduite de l'URL : `produit` et `projet` sont les accueils de section, `relecture` la
 * liste des documents à relire, `clarifications` leur tableau de bord ; `etape` à
 * `clarifier` n'existent que dans l'atelier, `reception` à `traitement` dans le cahier de réception, et
 * `parametres` (pas dans le site exporté). */
export type Route =
  | { page: "accueil" }
  | { page: "produit" }
  | { page: "projet" }
  | { page: "relecture" }
  | { page: "fichier"; chemin: string; ancre: string }
  | { page: "document"; id: string; ancre: string }
  | { page: "recherche"; q: string }
  | { page: "graphe"; centre: string }
  | { page: "atelier" }
  | { page: "etape"; etape: string }
  | { page: "clarification"; id: string }
  | { page: "clarifier"; sujet: string }
  | { page: "clarifications" }
  | { page: "reception" }
  | { page: "reception-page"; cle: string }
  | { page: "traitement" }
  | { page: "parametres" };

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
  if (chemin === "/produit" || chemin === "/produit/") return { page: "produit" };
  if (chemin === "/projet" || chemin === "/projet/") return { page: "projet" };
  if (chemin === "/projet/relecture") return { page: "relecture" };
  if (chemin.startsWith("/produit/")) return { page: "fichier", chemin: chemin.slice("/produit/".length), ancre };
  if (chemin.startsWith("/projet/")) return { page: "document", id: chemin.slice("/projet/".length), ancre };
  if (chemin === "/recherche") return { page: "recherche", q: params.get("q") ?? "" };
  if (chemin === "/graphe") return { page: "graphe", centre: params.get("centre") ?? "" };
  if (chemin === "/atelier") return { page: "atelier" };
  if (chemin.startsWith("/atelier/")) return { page: "etape", etape: chemin.slice("/atelier/".length) };
  if (chemin.startsWith("/clarification/")) return { page: "clarification", id: chemin.slice("/clarification/".length) };
  if (chemin === "/clarifications") return { page: "clarifications" };
  if (chemin.startsWith("/clarifier/")) return { page: "clarifier", sujet: chemin.slice("/clarifier/".length) };
  if (chemin === "/parametres") return { page: "parametres" };
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
const avecParcours = (p?: string) => (p && p !== "PARCOURS" ? `?parcours=${encodeURIComponent(p)}` : "");

/** Parcours de l'atelier désigné par l'URL courante (`PARCOURS` sans paramètre). */
export function parcoursDeUrl(): string {
  return lireUrl().params.get("parcours") ?? "PARCOURS";
}

/**
 * Fabrique des URL stables : `/` (accueil), `/produit` et `/projet` (accueils de section), `/projet/relecture`,
 * `/produit/<chemin>#<symbole>`, `/projet/<id>`, `/recherche`, `/graphe`,
 * précédées de `#` en mode statique ; pour l'atelier : `/atelier`, `/atelier/<étape>` (avec
 * `?parcours=` hors du parcours par défaut), `/clarifications`, `/clarification/<id>`, `/clarifier/<id>` ; pour la réception : `/reception`, `/reception/page/?cle=<clé>`, `/reception/traitement` ; `/parametres`.
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
  atelier: (parcours?: string) => `${prefixe}/atelier${avecParcours(parcours)}`,
  etape: (etape: string, parcours?: string) => `${prefixe}/atelier/${encodeURIComponent(etape)}${avecParcours(parcours)}`,
  clarifications: () => `${prefixe}/clarifications`,
  clarification: (id: string) => `${prefixe}/clarification/${encodeURIComponent(id)}`,
  clarifier: (sujet: string) => `${prefixe}/clarifier/${encodeURIComponent(sujet)}`,
  reception: () => `${prefixe}/reception`,
  // La clé d'une page est souvent une route (`/configurer`) : elle voyage en paramètre.
  pageReception: (cle: string) => `${prefixe}/reception/page/?cle=${encodeURIComponent(cle)}`,
  traitement: () => `${prefixe}/reception/traitement`,
  parametres: () => `${prefixe}/parametres`,
};
