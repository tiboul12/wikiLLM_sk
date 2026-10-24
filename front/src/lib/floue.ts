// Recherche floue au fil de la frappe (EF-172, REC-087, REC-113), sur le modèle de « Trouver un nœud » du graphe :
// insensible à la casse et aux accents, l'ordre des mots est libre, une petite faute passe. Le score classe
// l'égalité exacte, puis le début d'identifiant, la fin de chemin, les mots tous présents, et enfin les lettres
// dans l'ordre (« bdrv9 » trouve « BDR-v9-1 »).
//
// @req EF-172

/** Un élément cherchable : identifiant, libellé, et texte secondaire facultatif (chemin, catégorie). */
export type Cherchable = { id: string; libelle: string; detail?: string };

/** Texte en minuscules sans accents ni ponctuation superflue, pour comparer. */
export function normaliser(t: string): string {
  return t
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .trim();
}

/** Les lettres de `q` apparaissent-elles dans `t`, dans l'ordre ? */
function sousSuite(q: string, t: string): boolean {
  let i = 0;
  for (const c of t) if (c === q[i] && ++i === q.length) return true;
  return q.length === 0;
}

/** Deux mots à une faute près : une lettre en trop, en moins, différente, ou deux lettres voisines inversées. */
function presque(a: string, b: string): boolean {
  if (Math.abs(a.length - b.length) > 1 || a.length < 4) return false;
  if (a.length === b.length) {
    const d = [...a].map((c, i) => (c === b[i] ? -1 : i)).filter((i) => i >= 0);
    if (d.length === 2 && d[1] === d[0] + 1 && a[d[0]] === b[d[1]] && a[d[1]] === b[d[0]]) return true;
  }
  let i = 0;
  let j = 0;
  let ecarts = 0;
  while (i < a.length && j < b.length) {
    if (a[i] === b[j]) {
      i++;
      j++;
      continue;
    }
    if (++ecarts > 1) return false;
    if (a.length > b.length) i++;
    else if (b.length > a.length) j++;
    else {
      i++;
      j++;
    }
  }
  return ecarts + (a.length - i) + (b.length - j) <= 1;
}

/** Score d'un élément pour la requête ; 0 : il ne correspond pas. */
export function score(q: string, e: Cherchable): number {
  const r = normaliser(q);
  if (!r) return 0;
  const id = normaliser(e.id);
  const lib = normaliser(e.libelle);
  const tout = `${id} ${lib} ${normaliser(e.detail ?? "")}`;
  if (id === r || lib === r) return 100;
  if (id.startsWith(r)) return 80;
  if (id.endsWith("/" + r) || id.endsWith("::" + r)) return 70;
  if (lib.startsWith(r)) return 60;
  const mots = r.split(/\s+/);
  const motsTexte = tout.split(/[\s/:.\-_·,()«»]+/).filter(Boolean);
  if (mots.every((m) => tout.includes(m))) return 40 + Math.min(9, mots.length);
  if (mots.every((m) => tout.includes(m) || motsTexte.some((w) => presque(m, w)))) return 25;
  const compact = r.replace(/[\s\-_]/g, "");
  if (compact.length >= 3 && sousSuite(compact, id.replace(/[\s\-_]/g, ""))) return 15;
  return 0;
}

/** Les `n` meilleurs éléments pour la requête, du plus pertinent au moins pertinent. */
export function chercher<T extends Cherchable>(q: string, elements: T[], n = 12): T[] {
  return elements
    .map((e) => ({ e, s: score(q, e) }))
    .filter((x) => x.s > 0)
    .sort((a, b) => b.s - a.s || a.e.id.localeCompare(b.e.id, "fr", { numeric: true }))
    .slice(0, n)
    .map((x) => x.e);
}
