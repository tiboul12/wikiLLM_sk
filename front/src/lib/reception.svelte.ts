// État du cahier de réception (SPEC-23) : chargé depuis `/api/reception/etat`, rechargé après chaque
// écriture. Le serveur du projet le sert toujours (ADR-0012) ; le site exporté ne l'a pas.
//
// @req EF-123, EF-124
import type { EtatReception } from "../types/EtatReception";
import type { Remarque } from "../types/Remarque";
import { statique } from "./statique";
import { adapterHote } from "./atelier.svelte";

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

/**
 * Lien vers une page de l'application reçue : adresse saisie + route ; null sans l'une ou l'autre.
 * Une adresse de boucle locale prend l'hôte du navigateur (même règle que les liens wiki ↔ atelier).
 */
export function lienApplication(route: string | null | undefined): string | null {
  const base = reception.etat?.url_application;
  if (!base || !route) return null;
  return adapterHote(base) + (route.startsWith("/") ? route : "/" + route);
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
