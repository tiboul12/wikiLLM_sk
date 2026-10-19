// Documents de gestion en tableau à colonnes, façon Trello (EF-143, REC-017.2) : un plan d'après son
// tableau de suivi (à faire, en cours, fait), le backlog d'après ses sections et ses cases à cocher.
// Rien n'est stocké à part : les colonnes sont relues dans le Markdown à chaque affichage.
//
// @req EF-143

/** Une carte : un élément du suivi ou du backlog ; `detail` est du Markdown. */
export type Carte = { titre: string; detail: string; fait: boolean; date: string };
/** Une colonne du tableau. */
export type Colonne = { nom: string; cartes: Carte[] };

const cellules = (ligne: string) =>
  ligne
    .trim()
    .replace(/^\||\|$/g, "")
    .split(/(?<!\\)\|/)
    .map((c) => c.trim());

/**
 * Colonnes d'un plan : le premier tableau qui suit un titre « Suivi » ; une ligne marquée ✅ (ou « terminé »)
 * est faite, 🟡 / « en cours » est en cours, le reste est à faire. `null` si le plan n'a pas de suivi.
 */
export function colonnesPlan(md: string): Colonne[] | null {
  const lignes = md.split("\n");
  const titre = lignes.findIndex((l) => /^#{1,4}\s.*suivi/i.test(l));
  if (titre < 0) return null;
  const debut = lignes.findIndex((l, i) => i > titre && l.trim().startsWith("|"));
  if (debut < 0) return null;
  const entete = cellules(lignes[debut]).map((c) => c.toLowerCase());
  const iStatut = Math.max(entete.findIndex((c) => /statut|état|etat/.test(c)), 1);
  const iDate = entete.findIndex((c) => c.includes("date"));
  const col: Record<string, Carte[]> = { "À faire": [], "En cours": [], Fait: [] };
  for (let i = debut + 2; i < lignes.length && lignes[i].trim().startsWith("|"); i++) {
    const c = cellules(lignes[i]);
    const statut = c[iStatut] ?? "";
    const etat = /✅|termin|^fait/i.test(statut) ? "Fait" : /🟡|🔄|en cours/i.test(statut) ? "En cours" : "À faire";
    const detail = statut.replace(/^(✅|🟡|🔄|❌|⏳)\s*/u, "").replace(/^(à faire|terminée?)\s*:?\s*/i, "");
    col[etat].push({ titre: c[0] ?? "", detail, fait: etat === "Fait", date: iDate >= 0 ? (c[iDate] ?? "") : "" });
  }
  return Object.entries(col).map(([nom, cartes]) => ({ nom, cartes }));
}

/**
 * Colonnes du backlog : une par section (titre `##` ou `###`) qui contient des cases à cocher ; une carte
 * par case, son texte en gras pour titre. `null` s'il n'y a aucune case.
 */
export function colonnesBacklog(md: string): Colonne[] | null {
  const colonnes: Colonne[] = [];
  let courante: Colonne | null = null;
  for (const l of md.split("\n")) {
    const t = /^#{2,3}\s+(.*)/.exec(l);
    if (t) {
      courante = { nom: t[1].trim(), cartes: [] };
      colonnes.push(courante);
      continue;
    }
    const c = /^\s*[-*]\s+\[([ xX])\]\s+(.*)/.exec(l);
    if (!c || !courante) continue;
    const gras = /^\*\*(.+?)\*\*\s*:?\s*(.*)/.exec(c[2]);
    courante.cartes.push({ titre: gras ? gras[1] : c[2], detail: gras ? gras[2] : "", fait: c[1] !== " ", date: "" });
  }
  const pleines = colonnes.filter((c) => c.cartes.length);
  return pleines.length ? pleines : null;
}
