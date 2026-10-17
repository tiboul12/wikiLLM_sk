// Client de l'API en lecture seule de `wikik serve` (SPEC-09). Types générés depuis Rust (ts-rs).
import type { Projet } from "../types/Projet";
import type { PageFichier } from "../types/PageFichier";
import type { PageDocument } from "../types/PageDocument";
import type { Graphe } from "../types/Graphe";
import type { Resultat } from "../types/Resultat";

export type Diagnostic = {
  fichier: string;
  ligne: number;
  code: string;
  symbole: string | null;
  severite: string;
  message: string;
};
export type RapportCheck = {
  mode_reference: string;
  diagnostics: Diagnostic[];
  resume: { erreurs: number; avertissements: number; infos: number };
};

async function lire<T>(chemin: string, params: Record<string, string | number | undefined> = {}): Promise<T> {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) if (v !== undefined && v !== "") q.set(k, String(v));
  const url = q.toString() ? `${chemin}?${q}` : chemin;
  const r = await fetch(url);
  if (!r.ok) {
    let message = `${r.status}`;
    try {
      message = (await r.json()).erreur ?? message;
    } catch {
      /* corps non JSON */
    }
    throw new Error(message);
  }
  return r.json() as Promise<T>;
}

export const api = {
  projet: () => lire<Projet>("/api/projet"),
  fichier: (chemin: string) => lire<PageFichier>("/api/fichier", { chemin }),
  document: (id: string) => lire<PageDocument>("/api/document", { id }),
  graphe: (centre?: string, profondeur?: number) => lire<Graphe>("/api/graphe", { centre, profondeur }),
  cherche: (q: string) => lire<Resultat[]>("/api/cherche", { q }),
  check: () => lire<RapportCheck>("/api/check"),
};
