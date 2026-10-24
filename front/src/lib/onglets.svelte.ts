// Onglets de pages, à la VS Code (EF-151, EF-169, REC-069) : chaque page ouverte a son onglet au-dessus du contenu.
// Une navigation simple ouvre la page dans l'onglet d'aperçu (titre en italique), qui est remplacé à la
// navigation suivante : on parcourt sans accumuler d'onglets, et les onglets fixés ne sont jamais écrasés.
// L'aperçu se fixe par un double-clic sur l'onglet ou dès qu'on écrit dans la page ; Ctrl+clic, clic du
// milieu ou « ouvrir dans un onglet » (graphe) ouvrent un onglet fixé. Une page déjà ouverte réactive son
// onglet au lieu d'en ouvrir un second. ↗ détache l'onglet dans une fenêtre du navigateur, sans barre
// d'onglets. Les onglets sont gardés par ce navigateur (rechargement compris).
//
// @req EF-151, EF-169
import { aller, adresseCourante, type Route } from "./routeur.svelte";
import type { Projet } from "../types/Projet";

/** Un onglet : l'adresse de sa page, le titre affiché et, pour l'onglet d'aperçu (un seul au plus), `apercu`. */
export type Onglet = { url: string; titre: string; apercu?: boolean };

/** Clé de stockage des onglets dans ce navigateur. */
const CLE = "wikik.onglets";

/** Onglets gardés par ce navigateur ; liste vide si rien n'est gardé ou lisible (la page courante prend alors un onglet). */
function lire(): { liste: Onglet[]; actif: number } {
  try {
    const v = JSON.parse(localStorage.getItem(CLE) ?? "null");
    if (v && Array.isArray(v.liste) && v.liste.length && typeof v.actif === "number") return { liste: v.liste, actif: Math.min(v.actif, v.liste.length - 1) };
  } catch {
    /* stockage absent ou illisible : un seul onglet */
  }
  return { liste: [], actif: 0 };
}

/**
 * Fenêtre détachée : ouverte par ↗ avec `?detache=1`, elle n'a pas de barre d'onglets (le paramètre est
 * retiré de l'adresse au chargement).
 */
export const detachee = (() => {
  const p = new URLSearchParams(location.search);
  if (p.get("detache") !== "1") return false;
  p.delete("detache");
  const q = p.toString();
  history.replaceState(null, "", location.pathname + (q ? `?${q}` : "") + location.hash);
  return true;
})();

/** Onglets ouverts et onglet actif. Une fenêtre détachée ne les lit ni ne les écrit. */
export const onglets = $state(detachee ? { liste: [] as Onglet[], actif: 0 } : lire());

/** Garde les onglets ; une fenêtre détachée n'en a pas et n'écrit rien. */
function garder() {
  if (detachee) return;
  try {
    localStorage.setItem(CLE, JSON.stringify({ liste: onglets.liste, actif: onglets.actif }));
  } catch {
    /* sans stockage, les onglets valent pour la session */
  }
}

/** Adresse sans son ancre : une page et un symbole de cette page partagent l'onglet. */
const sansAncre = (url: string) => url.split("#")[0];

/**
 * Suit la page affichée (à chaque changement de route). Si l'onglet actif l'affiche déjà, il prend son adresse et
 * son titre ; si un autre onglet l'affiche, il devient actif ; sinon la page va dans l'onglet d'aperçu, remplacé, ou
 * créé juste après l'onglet actif. Aucune navigation n'écrase un onglet fixé (EF-169).
 */
export function suivre(titre: string) {
  if (detachee) return;
  const url = adresseCourante();
  const actif = onglets.liste[onglets.actif];
  if (actif && sansAncre(actif.url) === sansAncre(url)) {
    onglets.liste[onglets.actif] = { ...actif, url, titre };
  } else {
    const deja = onglets.liste.findIndex((o) => sansAncre(o.url) === sansAncre(url));
    if (deja >= 0) {
      onglets.actif = deja;
      onglets.liste[deja] = { ...onglets.liste[deja], url, titre };
    } else {
      const ap = onglets.liste.findIndex((o) => o.apercu);
      if (ap >= 0) {
        onglets.liste[ap] = { url, titre, apercu: true };
        onglets.actif = ap;
      } else {
        const rang = onglets.liste.length ? onglets.actif + 1 : 0;
        onglets.liste.splice(rang, 0, { url, titre, apercu: true });
        onglets.actif = rang;
      }
    }
  }
  garder();
}

/** Fixe l'onglet `i` (double-clic) : la navigation suivante ne le remplacera plus. */
export function fixer(i: number) {
  const o = onglets.liste[i];
  if (!o?.apercu) return;
  onglets.liste[i] = { url: o.url, titre: o.titre };
  garder();
}

/** Fixe l'onglet actif : appelé après chaque écriture dans la page (édition, validation). */
export function fixerActif() {
  fixer(onglets.actif);
}

/** Ouvre `url` dans un nouvel onglet fixé, juste après l'onglet actif, et l'affiche. */
export function ouvrir(url: string) {
  if (detachee) {
    window.open(url, "_blank");
    return;
  }
  const deja = onglets.liste.findIndex((o) => sansAncre(o.url) === sansAncre(url));
  if (deja >= 0) {
    fixer(deja);
    activer(deja);
    return;
  }
  onglets.liste.splice(onglets.actif + 1, 0, { url, titre: "…" });
  onglets.actif++;
  garder();
  aller(url);
}

/** Affiche l'onglet `i`. */
export function activer(i: number) {
  if (i < 0 || i >= onglets.liste.length) return;
  onglets.actif = i;
  garder();
  aller(onglets.liste[i].url);
}

/**
 * Ferme l'onglet `i` ; fermer le dernier ramène à l'accueil. Fermer l'onglet actif affiche son voisin (celui de droite,
 * ou de gauche s'il était le dernier) : sa page est rechargée, sinon le contenu de l'onglet fermé resterait affiché (REC-089).
 */
export function fermer(i: number) {
  const etaitActif = i === onglets.actif;
  onglets.liste.splice(i, 1);
  if (!onglets.liste.length) {
    onglets.actif = 0;
    garder();
    aller("/");
    return;
  }
  if (etaitActif) {
    activer(Math.min(i, onglets.liste.length - 1));
    return;
  }
  if (i < onglets.actif) onglets.actif--;
  garder();
}

/** Déplace l'onglet `de` à la place `vers` (glisser-déposer). */
export function deplacer(de: number, vers: number) {
  if (de === vers) return;
  const actif = onglets.liste[onglets.actif];
  const [o] = onglets.liste.splice(de, 1);
  onglets.liste.splice(vers, 0, o);
  onglets.actif = onglets.liste.indexOf(actif);
  garder();
}

/** Détache l'onglet `i` dans une fenêtre du navigateur, puis le ferme ici. */
export function detacher(i: number) {
  const o = onglets.liste[i];
  if (!o) return;
  const [chemin, ancre = ""] = o.url.split("#");
  const sep = chemin.includes("?") ? "&" : "?";
  window.open(`${chemin}${sep}detache=1${ancre ? "#" + ancre : ""}`, "_blank", "popup,width=1100,height=800");
  fermer(i);
}

/** Titre d'onglet d'une page : identifiant et titre d'un document, nom d'un fichier, nom de l'écran. */
export function titreDe(route: Route, projet: Projet | null): string {
  switch (route.page) {
    case "document": {
      const d = projet?.documents.find((x) => x.id === route.id);
      return d && d.titre !== d.id ? `${d.id} · ${d.titre}` : route.id;
    }
    case "fichier":
      return route.chemin.split("/").pop() + (route.ancre ? ` · ${route.ancre}` : "");
    case "recherche":
      return route.q ? `Recherche « ${route.q} »` : "Recherche";
    case "graphe":
      return route.centre ? `Graphe · ${route.centre}` : "Graphe";
    case "etape":
      return `Étape · ${route.etape}`;
    case "clarification":
      return route.id;
    case "clarifier":
      return `Clarifier ${route.sujet}`;
    case "reception-page":
      return `Réception · ${route.cle}`;
    default:
      return TITRES[route.page] ?? route.page;
  }
}

/** Titre d'onglet des pages sans titre propre, par genre de page.
 * @relu 2026-10-24 69724965
 */
const TITRES: Record<string, string> = {
  accueil: "Accueil",
  produit: "Code",
  projet: "Projet",
  relecture: "À relire",
  ateliers: "Design thinking",
  atelier: "Atelier",
  clarifications: "Clarifications",
  reception: "Cahier de réception",
  traitement: "Traitement",
  parametres: "Paramètres",
  git: "Historique git",
  publication: "Publication git",
};

