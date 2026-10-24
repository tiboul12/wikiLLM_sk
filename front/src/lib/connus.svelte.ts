// Identifiants connus du projet (documents, fichiers de code), chargés avec le projet : un document ou une
// doc de code qui les cite en obtient le lien sans l'écrire en [[...]] (REC-018.2, EF-143).
//
// @req EF-143
import type { Projet } from "../types/Projet";
import type { DocumentResume } from "../types/DocumentResume";
import { url } from "./routeur.svelte";

/** Ensembles remplacés d'un bloc au chargement (assignation : le rendu qui les lit se met à jour).
 * @relu 2026-10-22 55020bdc
 */
export const connus = $state({
  documents: new Set<string>(),
  fichiers: new Set<string>(),
  minuscules: new Map<string, string>(),
  /** Tests d'exigence du projet (cascade, EF-160). */
  tests: [] as DocumentResume[],
});

/** Retient les documents et les fichiers du projet chargé.
 * @relu 2026-10-22 2b15c51e
 */
export function retenir(p: Projet) {
  connus.documents = new Set(p.documents.map((d) => d.id));
  connus.minuscules = new Map(p.documents.map((d) => [d.id.toLowerCase(), d.id]));
  connus.fichiers = new Set(p.fichiers.map((f) => f.chemin));
  connus.tests = p.documents.filter((d) => d.genre === "test");
}

/**
 * Lien d'un texte cité : identifiant de document (`EF-140`, `PLAN-v4`), chemin de fichier, `chemin::Symbole`
 * ou `chemin:ligne` ; `null` si rien de connu ne correspond. Sert aussi de résolveur [[...]] hors des
 * pages de document (`[[plan-v1]]` : identifiant sans tenir compte de la casse).
 */
export function autoLien(texte: string): string | null {
  const t = texte.trim();
  if (connus.documents.has(t)) return url.document(t);
  const id = connus.minuscules.get(t.toLowerCase());
  if (id) return url.document(id);
  if (connus.fichiers.has(t)) return url.fichier(t);
  const s = /^(.+?)::(.+)$/.exec(t);
  if (s && connus.fichiers.has(s[1])) return url.fichier(s[1], s[2]);
  const l = /^(.+?):\d+(?:-\d+)?$/.exec(t);
  if (l && connus.fichiers.has(l[1])) return url.fichier(l[1]);
  return null;
}
