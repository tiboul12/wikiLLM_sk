// Client de l'API de lecture du wiki (SPEC-09), servie par `wikik`. Types générés depuis Rust (ts-rs).
//
// @req EF-040
import type { Projet } from "../types/Projet";
import type { PageFichier } from "../types/PageFichier";
import type { PageDocument } from "../types/PageDocument";
import type { Graphe } from "../types/Graphe";
import type { Resultat } from "../types/Resultat";
import { statique, lireDonnee, cheminSur, nomSur, grapheLocal, chercher } from "./statique";

/** Diagnostic de `wikik check` (même forme que le JSON de SPEC-04). */
export type Diagnostic = {
  fichier: string;
  ligne: number;
  code: string;
  symbole: string | null;
  severite: string;
  message: string;
};
/** Rapport complet de `wikik check`. */
export type RapportCheck = {
  mode_reference: string;
  diagnostics: Diagnostic[];
  resume: { erreurs: number; avertissements: number; infos: number };
};

/** GET JSON avec paramètres (les vides sont omis) ; une réponse en erreur lève le message `{erreur}` du serveur. */
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

/**
 * Une fonction par route de l'API ; une réponse en erreur lève une exception avec le message du serveur.
 * En mode statique (site exporté), les mêmes fonctions lisent les fichiers de `donnees/`.
 */
export const api = statique
  ? {
      projet: () => lireDonnee<Projet>("projet.json"),
      fichier: (chemin: string) => lireDonnee<PageFichier>(`fichiers/${cheminSur(chemin)}.json`),
      document: (id: string) => lireDonnee<PageDocument>(`documents/${nomSur(id)}.json`),
      graphe: (centre?: string, profondeur?: number) =>
        centre ? grapheLocal(centre, profondeur ?? 1) : lireDonnee<Graphe>("graphe.json"),
      cherche: (q: string) => chercher(q),
      check: () => lireDonnee<RapportCheck>("check.json"),
    }
  : {
  projet: () => lire<Projet>("/api/projet"),
  fichier: (chemin: string) => lire<PageFichier>("/api/fichier", { chemin }),
  document: (id: string) => lire<PageDocument>("/api/document", { id }),
  graphe: (centre?: string, profondeur?: number) => lire<Graphe>("/api/graphe", { centre, profondeur }),
  cherche: (q: string) => lire<Resultat[]>("/api/cherche", { q }),
  check: () => lire<RapportCheck>("/api/check"),
};
