// État de l'atelier (SPEC-20) : chargé depuis `/api/atelier/etat`, rechargé après chaque écriture.
//
// Le serveur du projet sert toujours l'atelier (ADR-0012) ; le site exporté ne l'a pas.
//
// @req EF-100, EF-101
import type { EtatAtelier } from "../types/EtatAtelier";
import type { ArtefactResume } from "../types/ArtefactResume";
import { statique } from "./statique";

/**
 * État partagé : `actif` quand le serveur sert l'atelier, `etat` (null sinon),
 * `occupe` : appels au LLM en cours.
 */
export const atelier = $state({
  actif: false,
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

/** Charge l'état de l'atelier ; sans effet dans le site exporté. */
export async function detecter(): Promise<boolean> {
  if (statique) return false;
  try {
    atelier.etat = await lireEtat();
    atelier.actif = true;
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

/** Lien vers une page de l'atelier ; null dans le site exporté (pas d'atelier). */
export function lienAtelier(chemin: string): string | null {
  return atelier.actif ? chemin : null;
}

/**
 * Une adresse de boucle locale (`127.0.0.1`, `localhost`, valeur par défaut de `wikik.toml`) prend l'hôte
 * par lequel le navigateur joint déjà le serveur, en gardant son port : les liens wiki ↔ atelier marchent
 * depuis un autre poste sans rien configurer. Une adresse explicite (nom, autre IP) est gardée telle quelle.
 */
export function adapterHote(adresse: string): string {
  try {
    const u = new URL(adresse);
    if (["127.0.0.1", "localhost", "[::1]"].includes(u.hostname) && location.hostname) u.hostname = location.hostname;
    return u.toString().replace(/\/$/, "");
  } catch {
    return adresse;
  }
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
