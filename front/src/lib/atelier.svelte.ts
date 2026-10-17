// État de l'atelier (SPEC-20) : chargé depuis `/api/atelier/etat`, rechargé après chaque écriture.
//
// Le mode atelier est donné par `/api/mode` (servie par `wikik serve` et `wikik atelier`, absente du
// site exporté).
//
// @req EF-100, EF-101
import type { EtatAtelier } from "../types/EtatAtelier";
import type { ArtefactResume } from "../types/ArtefactResume";
import { statique } from "./statique";

/**
 * État partagé : `actif` dans l'atelier, `url` de l'atelier vue du wiki, `etat` (null hors atelier),
 * `occupe` : appels au LLM en cours.
 */
export const atelier = $state({
  actif: false,
  /** Adresse de l'atelier vue depuis `wikik serve` (null sans atelier, ou dans le site exporté). */
  url: null as string | null,
  etat: null as EtatAtelier | null,
  occupe: 0,
  depuis: 0,
  erreur: "",
});

async function lireEtat(): Promise<EtatAtelier> {
  const r = await fetch("/api/atelier/etat");
  if (!r.ok) throw new Error(String(r.status));
  return r.json();
}

/** Détecte le mode atelier et charge l'état ; sans effet en mode statique ou sur `wikik serve`.
 * @relu 2026-10-17 c5c12988
 */
export async function detecter(): Promise<boolean> {
  if (statique) return false;
  try {
    const mode = await fetch("/api/mode").then((r) => r.json());
    if (mode.atelier) {
      atelier.etat = await lireEtat();
      atelier.actif = true;
    }
  } catch {
    atelier.actif = false;
  }
  return atelier.actif;
}

/** Relit l'état de l'atelier (après une écriture). */
export async function recharger() {
  atelier.etat = await lireEtat();
}

/** POST JSON vers une route de l'atelier ; l'état est rechargé ensuite ; l'erreur du serveur est levée. */
export async function ecrire<T = unknown>(route: string, corps: unknown, llm = false): Promise<T> {
  if (llm) {
    if (!atelier.occupe) atelier.depuis = Date.now();
    atelier.occupe++;
  }
  atelier.erreur = "";
  try {
    const r = await fetch(`/api/atelier/${route}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(corps),
    });
    const v = await r.json().catch(() => ({}));
    if (!r.ok) throw new Error(v.erreur ?? `${r.status}`);
    await recharger();
    return v as T;
  } catch (e) {
    atelier.erreur = e instanceof Error ? e.message : String(e);
    throw e;
  } finally {
    if (llm) atelier.occupe--;
  }
}

/**
 * Lien vers une page de l'atelier : relatif quand l'atelier sert lui-même le wiki, absolu depuis
 * `wikik serve` ; null sans atelier.
 */
export function lienAtelier(chemin: string): string | null {
  if (atelier.actif) return chemin;
  return atelier.url ? atelier.url + chemin : null;
}

/** Artefacts d'un type, dans l'ordre des identifiants. */
export function deType(genre: string | null | undefined): ArtefactResume[] {
  return (atelier.etat?.artefacts ?? []).filter((a) => a.type === genre);
}

/** Artefact par identifiant. */
export function artefact(id: string | null | undefined): ArtefactResume | undefined {
  return (atelier.etat?.artefacts ?? []).find((a) => a.id === id);
}

/** Texte d'un champ de frontmatter. */
export function champ(a: ArtefactResume | undefined, cle: string): string {
  const v = a?.frontmatter?.[cle];
  return v === undefined || v === null ? "" : typeof v === "string" ? v : JSON.stringify(v);
}

/** Libellés des phases des fiches. */
export const PHASES: Record<string, string> = {
  cadrage: "Cadrage",
  ideation: "Idéation",
  raffinage: "Raffinage",
  iteration: "Itération",
  scenarisation: "Scénarisation",
  bilan: "Bilan",
  specification: "Spécification",
  planification: "Planification",
  rappel: "Rappel",
};

/** Libellés des états d'une étape. */
export const ETATS: Record<string, string> = {
  a_faire: "à faire",
  en_cours: "en cours",
  faite: "faite",
  sautee: "sautée",
};
