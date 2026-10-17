// Mode statique (site exporté par `wikik export`, ADR-0008) : pages lues dans `donnees/`, recherche et
// graphes locaux calculés dans le navigateur.
//
// @req EF-080

import type { Graphe } from "../types/Graphe";
import type { EntreeRecherche } from "../types/EntreeRecherche";
import type { Resultat } from "../types/Resultat";

/** Vrai si la page a été exportée en site statique (balise posée par `wikik export`). */
export const statique = !!document.querySelector('meta[name="wikik-statique"]');

/** Nom de fichier sûr, identique à `nom_sur` côté Rust : `~` + octets UTF-8 hors `[A-Za-z0-9._-]`. */
export function nomSur(t: string): string {
  let s = "";
  for (const c of t) {
    if (/[A-Za-z0-9._-]/.test(c)) s += c;
    else for (const b of new TextEncoder().encode(c)) s += "~" + b.toString(16).padStart(2, "0");
  }
  return s;
}

/** Chemin de fichier sûr, segment par segment. */
export function cheminSur(chemin: string): string {
  return chemin.split("/").map(nomSur).join("/");
}

/** Lit un fichier de `donnees/` ; une absence devient une erreur lisible. */
export async function lireDonnee<T>(relatif: string): Promise<T> {
  const r = await fetch(`./donnees/${relatif}`);
  if (!r.ok) throw new Error(r.status === 404 ? "introuvable dans le site exporté" : String(r.status));
  return r.json() as Promise<T>;
}

let grapheComplet: Promise<Graphe> | null = null;
let indexRecherche: Promise<EntreeRecherche[]> | null = null;

/** Graphe local autour de `centre`, mêmes règles que `wiki::graphe` (profondeur 1 à 3, 300 arêtes au plus). */
export async function grapheLocal(centre: string, profondeur: number): Promise<Graphe> {
  grapheComplet ??= lireDonnee<Graphe>("graphe-complet.json");
  const g = await grapheComplet;
  const vus = new Set([centre]);
  let frontiere = [centre];
  const aretes = new Map<string, Graphe["aretes"][number]>();
  for (let i = 0; i < Math.min(Math.max(profondeur, 1), 3) && aretes.size <= 300; i++) {
    const suivante: string[] = [];
    for (const n of frontiere)
      for (const a of g.aretes)
        if (a.source === n || a.cible === n) {
          aretes.set(`${a.source}\u0000${a.cible}\u0000${a.genre}`, a);
          for (const x of [a.source, a.cible]) if (!vus.has(x)) (vus.add(x), suivante.push(x));
        }
    frontiere = suivante;
  }
  const noeuds = g.noeuds.filter((n) => vus.has(n.id)).map((n) => ({ ...n, centre: n.id === centre }));
  return { noeuds, aretes: [...aretes.values()] };
}

const normaliser = (t: string) => t.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");

/** Recherche : tous les mots présents (sans casse ni accents), titres d'abord, 30 résultats. */
export async function chercher(q: string): Promise<Resultat[]> {
  indexRecherche ??= lireDonnee<EntreeRecherche[]>("recherche.json");
  const mots = normaliser(q).split(/[^\p{L}\p{N}_]+/u).filter(Boolean);
  if (!mots.length) return [];
  const notes: [number, EntreeRecherche, number][] = [];
  for (const e of await indexRecherche) {
    const titre = normaliser(e.titre);
    const texte = normaliser(e.texte);
    if (!mots.every((m) => titre.includes(m) || texte.includes(m))) continue;
    const note = mots.reduce((n, m) => n + (titre.includes(m) ? 10 : 0) + texte.split(m).length - 1, 0);
    notes.push([note, e, texte.indexOf(mots[0])]);
  }
  notes.sort((a, b) => b[0] - a[0] || a[1].cle.localeCompare(b[1].cle));
  return notes.slice(0, 30).map(([, e, p]) => ({
    cle: e.cle,
    genre: e.genre,
    titre: e.titre,
    fichier: e.fichier,
    extrait: p < 0 ? e.texte.slice(0, 160) : (p > 60 ? "…" : "") + e.texte.slice(Math.max(0, p - 60), p + 100) + "…",
  }));
}
