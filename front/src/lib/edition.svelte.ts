// Édition depuis l'interface (EF-157 à EF-159, ADR-0016) : statut et texte des documents, annuler et
// rétablir (Ctrl+Z, Ctrl+Y), favoris, et ce qui attend une décision de l'utilisateur. Après une écriture,
// `edition.revision` avance : l'application recharge le projet et redessine la page.
//
// @req EF-157, EF-158, EF-159
import type { DocumentResume } from "../types/DocumentResume";
import { poster } from "./terminal.svelte";
import { statique } from "./statique";

/** `revision` : avance après chaque écriture ; `message` : bandeau bref (« Annulé : … »). */
export const edition = $state({ revision: 0, message: "", erreur: false });

let minuterie: ReturnType<typeof setTimeout> | undefined;
/** Montre un message bref en bas de l'écran. */
export function signaler(message: string, erreur = false) {
  edition.message = message;
  edition.erreur = erreur;
  clearTimeout(minuterie);
  minuterie = setTimeout(() => (edition.message = ""), erreur ? 6000 : 3500);
}

/** Recharge le projet et redessine la page (après une écriture). */
export function rafraichir() {
  edition.revision++;
}

/** Change le statut d'un document (« Valider » de la relecture). */
export async function changerStatut(id: string, statut: string) {
  await poster("/api/document/statut", { id, statut });
  signaler(`${id} : ${statut} (Ctrl+Z pour annuler)`);
  rafraichir();
}

/** Texte complet d'un document, pour l'éditeur. */
export async function sourceDocument(id: string): Promise<{ chemin: string; texte: string }> {
  const r = await fetch(`/api/document/source?id=${encodeURIComponent(id)}`);
  const v = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(v.erreur ?? `${r.status}`);
  return v;
}

/** Remplace le texte d'un document. */
export async function ecrireDocument(id: string, texte: string) {
  await poster("/api/document/ecrire", { id, texte });
  signaler(`${id} enregistré (Ctrl+Z pour annuler)`);
  rafraichir();
}

type Bilan = { libelle: string; fichiers: string[] } | null;

/** Annule (ou rétablit) la dernière écriture faite depuis wikik. */
export async function annulerOuRetablir(retablir = false) {
  if (statique) return;
  try {
    const b = await poster<Bilan>(retablir ? "/api/retablir" : "/api/annuler", {});
    if (!b) return signaler(retablir ? "Rien à rétablir." : "Rien à annuler.");
    signaler(`${retablir ? "Rétabli" : "Annulé"} : ${b.libelle} (${b.fichiers.join(", ")})`);
    rafraichir();
  } catch (e) {
    signaler(String(e instanceof Error ? e.message : e), true);
  }
}

/** Ctrl+Z / Ctrl+Y (Ctrl+Maj+Z) hors des champs de saisie, où le navigateur garde sa propre annulation. */
export function raccourcis(e: KeyboardEvent) {
  if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
  const cible = e.target as HTMLElement | null;
  if (cible?.closest("input, textarea, select, [contenteditable=''], [contenteditable='true'], .xterm")) return;
  const k = e.key.toLowerCase();
  if (k === "z" && !e.shiftKey) {
    e.preventDefault();
    annulerOuRetablir(false);
  } else if (k === "y" || (k === "z" && e.shiftKey)) {
    e.preventDefault();
    annulerOuRetablir(true);
  }
}

/**
 * Types de documents qui demandent une décision (REC-054) : seuls ceux-là, au statut proposé ou brouillon,
 * sont « à relire ». Plans, backlog, mesures et notes n'y vont pas.
 */
export const GENRES_A_RELIRE = ["besoin", "exigence", "exigence-nf", "adr", "spec", "test", "bdr-analyse", "bdr-definition", "cahier-des-charges"];

/** Documents à relire du projet. */
export function aRelire(docs: DocumentResume[]): DocumentResume[] {
  return docs.filter((d) => d.zone === "projet" && GENRES_A_RELIRE.includes(d.genre) && (d.statut === "proposé" || d.statut === "brouillon"));
}

/**
 * Tâches du backlog à la charge de l'utilisateur (REC-048) : les cases non cochées des sections marquées ✋
 * ou « à faire avec … ».
 */
export function tachesBacklog(md: string): string[] {
  const taches: string[] = [];
  let pourMoi = false;
  for (const l of md.split("\n")) {
    const t = /^#{2,3}\s+(.*)/.exec(l);
    if (t) {
      pourMoi = /✋|à faire avec|a faire avec|pour toi|ta charge/i.test(t[1]);
      continue;
    }
    const c = /^\s*[-*]\s+\[ \]\s+(.*)/.exec(l);
    if (c && pourMoi) taches.push(c[1].replace(/\*\*/g, ""));
  }
  return taches;
}

/** Une page mise en favori. */
export type Favori = { url: string; titre: string };

/** Favoris de cette machine (`.wikik/favoris.json`), chargés au démarrage. */
export const favoris = $state({ liste: [] as Favori[] });

/** Charge les favoris ; sans effet dans le site exporté. */
export async function chargerFavoris() {
  if (statique) return;
  try {
    favoris.liste = await (await fetch("/api/favoris")).json();
  } catch {
    favoris.liste = [];
  }
}

/** Vrai si la page est en favori. */
export function estFavori(url: string): boolean {
  return favoris.liste.some((f) => f.url === url);
}

/** Ajoute ou retire une page des favoris. */
export async function basculerFavori(url: string, titre: string) {
  favoris.liste = estFavori(url) ? favoris.liste.filter((f) => f.url !== url) : [...favoris.liste, { url, titre }];
  try {
    await poster("/api/favoris", $state.snapshot(favoris.liste));
  } catch (e) {
    signaler(String(e instanceof Error ? e.message : e), true);
  }
}
