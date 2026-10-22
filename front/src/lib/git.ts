// Graphe des branches de l'historique git (EF-166, REC-067), à la GitKraken : chaque commit est placé dans
// un couloir ; un couloir suit le premier parent, les autres parents ouvrent ou rejoignent un couloir. Le
// calcul se fait ligne par ligne, dans l'ordre de `git log --date-order` (un parent après ses enfants).
//
// @req EF-166
import type { CommitVue } from "../types/CommitVue";

/** Un trait d'une ligne du graphe, de (couloir, position) à (couloir, position) : 0 haut, 1 milieu, 2 bas. */
export type Trait = { de: number; a: number; depuis: 0 | 1; vers: 1 | 2; couleur: number };

/** Une ligne du graphe : le couloir du commit et les traits qui traversent la ligne. */
export type Ligne = { commit: CommitVue; couloir: number; traits: Trait[]; largeur: number };

/** Place chaque commit dans un couloir et calcule les traits de chaque ligne. */
export function couloirs(commits: CommitVue[]): Ligne[] {
  const lanes: (string | null)[] = [];
  const couleurs: number[] = [];
  let prochaineCouleur = 0;
  const libre = () => {
    const i = lanes.indexOf(null);
    return i >= 0 ? i : lanes.length;
  };
  const lignes: Ligne[] = [];
  for (const c of commits) {
    const avant = [...lanes];
    let col = lanes.indexOf(c.hash);
    if (col < 0) {
      col = libre();
      lanes[col] = c.hash;
      couleurs[col] = prochaineCouleur++;
    }
    const traits: Trait[] = [];
    // Ce qui entre par le haut : rejoint le commit, ou passe.
    avant.forEach((h, k) => {
      if (h === null || h === undefined) return;
      if (h === c.hash) traits.push({ de: k, a: col, depuis: 0, vers: 1, couleur: couleurs[k] });
    });
    // Les autres couloirs qui attendaient ce commit se ferment ici.
    lanes.forEach((h, k) => {
      if (k !== col && h === c.hash) lanes[k] = null;
    });
    const couleurCommit = couleurs[col];
    // Le premier parent déjà attendu dans un autre couloir (branche qui rejoint) : ce couloir se ferme.
    const p0 = c.parents[0];
    lanes[col] = p0 && lanes.some((h, k) => h === p0 && k !== col) ? null : (p0 ?? null);
    for (const p of c.parents.slice(1)) {
      if (lanes.includes(p)) continue;
      const j = libre();
      lanes[j] = p;
      couleurs[j] = prochaineCouleur++;
    }
    // Ce qui passe sans toucher le commit.
    avant.forEach((h, k) => {
      if (h === null || h === undefined || h === c.hash) return;
      const j = lanes[k] === h ? k : lanes.indexOf(h);
      if (j >= 0) traits.push({ de: k, a: j, depuis: 0, vers: 2, couleur: couleurs[k] });
    });
    // Du commit vers ses parents.
    c.parents.forEach((p, n) => {
      const j = lanes.indexOf(p);
      if (j >= 0) traits.push({ de: col, a: j, depuis: 1, vers: 2, couleur: n === 0 ? couleurCommit : couleurs[j] });
    });
    while (lanes.length && lanes[lanes.length - 1] === null) lanes.pop();
    lignes.push({ commit: c, couloir: col, traits, largeur: Math.max(lanes.length, avant.length, col + 1) });
  }
  return lignes;
}

/** Teintes des couloirs, lisibles en clair comme en sombre. */
export const TEINTES = ["#3b82f6", "#16a34a", "#d97706", "#9333ea", "#e11d48", "#0891b2", "#65a30d", "#c026d3", "#ea580c", "#0d9488"];
