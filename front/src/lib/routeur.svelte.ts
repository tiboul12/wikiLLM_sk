// Routeur minimal à l'API History : URL stables (SPEC-05 R7) sans dépendance.
//
// @req EF-041, EF-042
/** Page à afficher, déduite de l'URL. */
export type Route =
  | { page: "accueil" }
  | { page: "fichier"; chemin: string; ancre: string }
  | { page: "document"; id: string; ancre: string }
  | { page: "recherche"; q: string }
  | { page: "graphe"; centre: string };

function analyser(): Route {
  const chemin = decodeURIComponent(location.pathname);
  const ancre = decodeURIComponent(location.hash.slice(1));
  const params = new URLSearchParams(location.search);
  if (chemin.startsWith("/produit/")) return { page: "fichier", chemin: chemin.slice("/produit/".length), ancre };
  if (chemin.startsWith("/projet/")) return { page: "document", id: chemin.slice("/projet/".length), ancre };
  if (chemin === "/recherche") return { page: "recherche", q: params.get("q") ?? "" };
  if (chemin === "/graphe") return { page: "graphe", centre: params.get("centre") ?? "" };
  return { page: "accueil" };
}

/** Route courante, réactive : l'application se redessine quand elle change. */
export const routeur = $state({ route: analyser() as Route });

/** Navigue sans recharger la page ; ne fait rien si l'URL est déjà la bonne. */
export function aller(url: string) {
  if (url === location.pathname + location.search + location.hash) return;
  history.pushState(null, "", url);
  routeur.route = analyser();
  if (!location.hash) window.scrollTo(0, 0);
}

addEventListener("popstate", () => (routeur.route = analyser()));

// Les liens internes (href commençant par /) sont gérés sans rechargement.
addEventListener("click", (e) => {
  if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
  const a = (e.target as Element | null)?.closest?.("a");
  const href = a?.getAttribute("href");
  if (!a || !href || !href.startsWith("/") || href.startsWith("//") || a.target) return;
  e.preventDefault();
  aller(href);
});

/** Fabrique des URL stables : `/produit/<chemin>#<symbole>`, `/projet/<id>`, `/recherche`, `/graphe`. */
export const url = {
  fichier: (chemin: string, ancre?: string) =>
    `/produit/${chemin.split("/").map(encodeURIComponent).join("/")}${ancre ? "#" + encodeURIComponent(ancre) : ""}`,
  document: (id: string) => `/projet/${encodeURIComponent(id)}`,
  recherche: (q: string) => `/recherche?q=${encodeURIComponent(q)}`,
  graphe: (centre?: string) => (centre ? `/graphe?centre=${encodeURIComponent(centre)}` : "/graphe"),
};
