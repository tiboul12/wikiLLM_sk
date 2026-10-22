// Onglets de pages, à la VS Code (EF-151, REC-069) : chaque page ouverte a son onglet au-dessus du contenu.
// Une navigation simple remplace la page de l'onglet actif ; Ctrl+clic, clic du milieu ou « ouvrir dans
// un onglet » (graphe) en ouvre un nouveau ; ↗ détache l'onglet dans une fenêtre du navigateur, sans barre
// d'onglets. Les onglets sont gardés par ce navigateur (rechargement compris).
//
// @req EF-151
import { aller, adresseCourante, type Route } from "./routeur.svelte";
import type { Projet } from "../types/Projet";

/** Un onglet : l'adresse de sa page et le titre affiché. */
export type Onglet = { url: string; titre: string };

const CLE = "wikik.onglets";

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

function garder() {
  if (detachee) return;
  try {
    localStorage.setItem(CLE, JSON.stringify({ liste: onglets.liste, actif: onglets.actif }));
  } catch {
    /* sans stockage, les onglets valent pour la session */
  }
}

// Arrivée dans l'application (rechargement, lien collé) : la page rejoint son onglet s'il existe, sinon
// elle en prend un nouveau ; ensuite, chaque navigation remplace la page de l'onglet actif.
let arrivee = true;

/** Suit la page affichée : l'onglet actif prend son adresse et son titre (à chaque changement de route). */
export function suivre(titre: string) {
  if (detachee) return;
  const url = adresseCourante();
  if (arrivee) {
    arrivee = false;
    const i = onglets.liste.findIndex((o) => o.url === url);
    if (i >= 0) onglets.actif = i;
    else {
      onglets.liste.push({ url, titre });
      onglets.actif = onglets.liste.length - 1;
    }
  }
  onglets.liste[onglets.actif] = { url, titre };
  garder();
}

/** Ouvre `url` dans un nouvel onglet, juste après l'onglet actif, et l'affiche. */
export function ouvrir(url: string) {
  if (detachee) {
    window.open(url, "_blank");
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

/** Ferme l'onglet `i` ; fermer le dernier ramène à l'accueil. */
export function fermer(i: number) {
  onglets.liste.splice(i, 1);
  if (!onglets.liste.length) {
    onglets.actif = 0;
    garder();
    aller("/");
    return;
  }
  if (i < onglets.actif || onglets.actif >= onglets.liste.length) onglets.actif = Math.max(0, onglets.actif - 1);
  else if (i === onglets.actif) {
    activer(Math.min(i, onglets.liste.length - 1));
    return;
  }
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

const TITRES: Record<string, string> = {
  accueil: "Accueil",
  produit: "Code",
  projet: "Projet",
  relecture: "À relire",
  cascade: "Cascade",
  atelier: "Design thinking",
  clarifications: "Clarifications",
  reception: "Cahier de réception",
  traitement: "Traitement",
  parametres: "Paramètres",
  git: "Historique git",
};

