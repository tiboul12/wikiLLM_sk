// État du cahier de réception (SPEC-23) : chargé depuis `/api/reception/etat`, rechargé après chaque
// écriture. Le mode est donné par `/api/mode` (`reception: true` sous `wikik reception`).
//
// @req EF-123, EF-124
import type { EtatReception } from "../types/EtatReception";
import type { Remarque } from "../types/Remarque";
import { statique } from "./statique";

/** État partagé : `actif` sous `wikik reception`, `etat` (null sinon), `occupe` pendant un appel au LLM. */
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

/** Détecte le mode réception et charge l'état ; sans effet ailleurs. */
export async function detecterReception(): Promise<boolean> {
  if (statique) return false;
  try {
    const mode = await fetch("/api/mode").then((r) => r.json());
    if (mode.reception) {
      reception.etat = await lireEtat();
      reception.actif = true;
    }
  } catch {
    reception.actif = false;
  }
  return reception.actif;
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

/** Remarques d'une page. */
export function remarquesDe(cle: string): Remarque[] {
  return (reception.etat?.remarques ?? []).filter((r) => r.page === cle);
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
