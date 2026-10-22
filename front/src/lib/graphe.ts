// Graphe global gardé pour la session (EF-153, REC-045) : chargé une fois, en tâche de fond après
// l'arrivée, avec sa disposition calculée hors écran ; la page Graphe s'ouvre alors tout de suite. « ↻ »
// le recharge. La disposition est aussi gardée par ce navigateur, tant que le graphe ne change pas.
//
// @req EF-153
import cytoscape from "cytoscape";
import { api } from "./api";
import type { Graphe } from "../types/Graphe";

/** Position de chaque nœud, par identifiant. */
export type Positions = Record<string, { x: number; y: number }>;

/** Graphe global de la session : une seule requête, partagée par tous les onglets de cette fenêtre. */
let global: Promise<Graphe> | null = null;
/** Dispositions déjà calculées, par signature de graphe : un même graphe se redessine au même endroit. */
const dispositions = new Map<string, Positions>();

/** Graphe global, chargé au premier appel (ou rechargé avec `forcer`). */
export function grapheGlobal(forcer = false): Promise<Graphe> {
  if (!global || forcer) {
    global = api.graphe();
    global.catch(() => (global = null));
  }
  return global;
}

/** Empreinte d'un graphe : même nœuds et mêmes liens, même disposition. */
export function signature(g: Graphe): string {
  let h = 0;
  const texte = g.noeuds.map((n) => n.id).sort().join("|") + "#" + g.aretes.map((a) => a.source + ">" + a.cible).sort().join("|");
  for (let i = 0; i < texte.length; i++) h = (Math.imul(h, 31) + texte.charCodeAt(i)) | 0;
  return `${g.noeuds.length}-${g.aretes.length}-${h >>> 0}`;
}

/** Générateur pseudo-aléatoire à graine fixe : la disposition « cose » est la même à chaque calcul (REC-006). */
function graine(n: number) {
  return () => {
    n = (n + 0x6d2b79f5) | 0;
    let t = Math.imul(n ^ (n >>> 15), 1 | n);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Position de départ tirée de l'identifiant : même graphe, même dessin. */
function depart(id: string, i: number, n: number) {
  let h = 0;
  for (const c of id) h = (Math.imul(h, 31) + c.charCodeAt(0)) | 0;
  const a = (2 * Math.PI * i) / Math.max(n, 1);
  const r = 300 + (Math.abs(h) % 200);
  return { x: r * Math.cos(a), y: r * Math.sin(a) };
}

/**
 * Disposition d'un graphe, calculée sans écran (cytoscape headless) puis gardée : en mémoire pour la
 * session, et dans le navigateur pour la prochaine ouverture du même graphe.
 *
 * @pitfall `cose` tire ses positions avec `Math.random` : il est remplacé par un générateur à graine fixe
 * le temps du calcul (synchrone), sinon le dessin changerait à chaque ouverture.
 */
export function disposer(g: Graphe): Positions {
  const sig = signature(g);
  const deja = dispositions.get(sig) ?? lire(sig);
  if (deja) {
    dispositions.set(sig, deja);
    return deja;
  }
  const n = g.noeuds.length;
  const ordonnes = [...g.noeuds].sort((a, b) => a.id.localeCompare(b.id));
  const ids = new Set(ordonnes.map((x) => x.id));
  const cy = cytoscape({
    headless: true,
    styleEnabled: true,
    elements: [
      ...ordonnes.map((x, i) => ({ data: { id: x.id }, position: depart(x.id, i, n) })),
      ...g.aretes.filter((a) => ids.has(a.source) && ids.has(a.cible)).map((a, i) => ({ data: { id: `a${i}`, source: a.source, target: a.cible } })),
    ],
    style: [{ selector: "node", style: { width: 12, height: 12 } }],
  });
  const hasard = Math.random;
  Math.random = graine(n * 7919 + g.aretes.length);
  try {
    cy.layout({
      name: "cose",
      animate: false,
      randomize: false,
      nodeRepulsion: () => 26000,
      idealEdgeLength: () => 110,
      nodeOverlap: 40,
      componentSpacing: 140,
      gravity: 0.6,
      numIter: 1500,
    } as cytoscape.LayoutOptions).run();
  } finally {
    Math.random = hasard;
  }
  const pos: Positions = {};
  cy.nodes().forEach((x) => void (pos[x.id()] = { ...x.position() }));
  cy.destroy();
  dispositions.set(sig, pos);
  garder(sig, pos);
  return pos;
}

/** Clé de stockage de la dernière disposition du graphe global. */
const CLE = "wikik.graphe";
/** Disposition gardée par ce navigateur, si elle correspond à ce graphe (même signature). */
function lire(sig: string): Positions | null {
  try {
    const v = JSON.parse(localStorage.getItem(CLE) ?? "null");
    return v?.sig === sig ? (v.pos as Positions) : null;
  } catch {
    return null;
  }
}
/** Garde la disposition du graphe pour la prochaine ouverture ; une seule est gardée, la plus récente. */
function garder(sig: string, pos: Positions) {
  try {
    localStorage.setItem(CLE, JSON.stringify({ sig, pos }));
  } catch {
    /* stockage plein ou absent : la disposition vaut pour la session */
  }
}

/** Précharge le graphe global et sa disposition quand le navigateur est libre (appelé à l'arrivée). */
export function precharger() {
  const plus_tard = (f: () => void) => ("requestIdleCallback" in window ? requestIdleCallback(f, { timeout: 4000 }) : setTimeout(f, 1500));
  plus_tard(() => {
    grapheGlobal()
      .then((g) => plus_tard(() => disposer(g)))
      .catch(() => {});
  });
}
