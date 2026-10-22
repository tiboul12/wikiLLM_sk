// État du cahier de réception (SPEC-23) : chargé depuis `/api/reception/etat`, rechargé après chaque
// écriture. Le serveur du projet le sert toujours (ADR-0012) ; le site exporté ne l'a pas.
//
// @req EF-123, EF-124
import type { EtatReception } from "../types/EtatReception";
import type { Remarque } from "../types/Remarque";
import type { Page } from "../types/Page";
import { statique } from "./statique";
import { adapterHote } from "./atelier.svelte";
import { url } from "./routeur.svelte";

/** État partagé : `actif` quand le serveur sert le cahier, `etat` (null sinon), `occupe` pendant un appel au LLM. */
export const reception = $state({
  actif: false,
  etat: null as EtatReception | null,
  occupe: false,
  erreur: "",
});

async function lireEtat(): Promise<EtatReception> {
  const r = await fetch("/api/reception/etat");
  if (!r.ok) throw new Error(String(r.status));
  return r.json();
}

/** Charge l'état du cahier ; sans effet dans le site exporté. */
export async function detecterReception(): Promise<boolean> {
  if (statique) return false;
  try {
    reception.etat = await lireEtat();
    reception.actif = true;
  } catch {
    reception.actif = false;
  }
  return reception.actif;
}

/**
 * Lien vers la carte de réception de l'écran dont la route est `t` (`/clarifications`), ou null : une
 * piste ou une source qui cite un écran de l'application en obtient le lien (REC-071).
 */
export function lienRoute(t: string): string | null {
  const r = t.trim();
  if (!r.startsWith("/") || !reception.etat) return null;
  const p = reception.etat.pages.find((x) => x.route === r) ?? reception.etat.pages.find((x) => x.route && r.startsWith(x.route.replace(/[:<[{].*$/, "")) && x.route.length > 1);
  return p ? url.pageReception(p.cle) : null;
}

/** Relit l'état du cahier (après une écriture faite ailleurs, ou Ctrl+Z). */
export async function recharger() {
  reception.etat = await lireEtat();
}

/** POST JSON vers une route du cahier, puis rechargement de l'état ; l'erreur du serveur est levée. */
export async function ecrireReception<T = unknown>(route: string, corps: unknown = {}): Promise<T> {
  reception.erreur = "";
  const r = await fetch(`/api/reception/${route}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(corps),
  });
  const v = await r.json().catch(() => ({}));
  if (!r.ok) {
    reception.erreur = v.erreur ?? `${r.status}`;
    throw new Error(reception.erreur);
  }
  reception.etat = await lireEtat();
  return v as T;
}

/** Remarques à jour d'une page (sans les versions remplacées). */
export function remarquesDe(cle: string): Remarque[] {
  return actuelles().filter((r) => r.page === cle);
}

/** Remarques à jour : sans les versions remplacées par une modification (elles restent dans l'historique). */
export function actuelles(): Remarque[] {
  return (reception.etat?.remarques ?? []).filter((r) => r.statut !== "remplacee");
}

/** Adresse de l'application reçue : celle saisie, sinon celle déduite du projet (`/` : ce serveur). */
export function baseApplication(): string | null {
  const b = reception.etat?.url_application ?? reception.etat?.url_detectee ?? null;
  if (!b) return null;
  return b === "/" ? location.origin : adapterHote(b);
}

/**
 * Lien vers une page de l'application reçue : adresse + route ; sans route connue, l'accueil de
 * l'application ; null si aucune adresse n'est connue. Une adresse de boucle locale prend l'hôte du
 * navigateur (même règle que les liens wiki ↔ atelier).
 */
export function lienApplication(route: string | null | undefined): string | null {
  const base = baseApplication();
  if (!base) return null;
  return route ? base + (route.startsWith("/") ? route : "/" + route) : base + "/";
}

/** Libellés des genres de remarque. */
export const GENRES: Record<string, string> = { bug: "Bug", ergonomie: "Ergonomie", evolution: "Évolution", question: "Question" };
/** Libellés des gravités. */
export const GRAVITES: Record<string, string> = { bloquant: "Bloquant", genant: "Gênant", detail: "Détail" };
/** Libellés des statuts. */
export const STATUTS: Record<string, string> = { ouverte: "Ouverte", traitee: "Traitée", reportee: "Reportée", rejetee: "Rejetée" };
/** Origine d'une page de la carte, telle qu'affichée (vide pour la page « Général »). */
export const SOURCES: Record<string, string> = {
  route: "route",
  "fichier-route": "fichier de route",
  enum: "navigation par état",
  html: "page HTML",
  composant: "composant de page",
  llm: "proposée par le LLM",
  manuel: "",
};

/** Nœud de l'arborescence des pages : une page et celles qu'elle mène. */
export type Noeud = { page: Page; enfants: Noeud[] };

/**
 * Arborescence des pages de l'application (REC-026) : le parent d'une page à route est la page dont la
 * route est son plus long préfixe (`/` en tête) ; une page sans route va sous la page de son dossier
 * (`atelier` → `/atelier`), sinon sous l'accueil. « Général » reste en tête, hors arbre.
 */
export function arbrePages(pages: Page[]): Noeud[] {
  const noeuds = new Map(pages.map((p) => [p.cle, { page: p, enfants: [] as Noeud[] }]));
  const segments = (r: string) => r.split("/").filter(Boolean);
  const avecRoute = pages.filter((p) => p.route);
  const parentDe = (p: Page): Page | undefined => {
    if (p.cle === "general") return undefined;
    if (p.route) {
      const s = segments(p.route);
      return avecRoute
        .filter((q) => q !== p && segments(q.route!).length < s.length && segments(q.route!).every((x, i) => x === s[i]))
        .sort((a, b) => segments(b.route!).length - segments(a.route!).length)[0];
    }
    const dossier = p.groupe?.split("/").pop();
    return avecRoute.find((q) => dossier && segments(q.route!).length === 1 && segments(q.route!)[0] === dossier) ?? avecRoute.find((q) => q.route === "/");
  };
  const racines: Noeud[] = [];
  for (const p of pages) {
    const parent = parentDe(p);
    (parent ? noeuds.get(parent.cle)!.enfants : racines).push(noeuds.get(p.cle)!);
  }
  return racines;
}

/**
 * Page montrée par défaut : ni masquée, ni disparue du code sans remarque ouverte (un écran renommé ne
 * reste pas en double une fois ses remarques traitées ; elles restent lisibles dans le traitement).
 */
export function visible(p: Page): boolean {
  return !p.masque && !(p.disparue && !remarquesDe(p.cle).some((r) => r.statut === "ouverte"));
}
