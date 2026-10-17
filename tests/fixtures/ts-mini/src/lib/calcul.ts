// Fonctions de calcul partagées.
// Sert de fixture TypeScript (SPEC-15).

/**
 * Double une valeur ; sature à 1000.
 *
 * @param x valeur d'entrée
 * @returns le double, borné
 * @req EF-001
 */
export function doubler(x: number): number {
  return Math.min(x * 2, 1000);
}

/** Triple une valeur. */
export const tripler = (x: number): number => x * 3;

function interne(): number {
  return 1;
}

/** Forme géométrique. */
export interface Forme {
  aire(): number;
}

/** Couleur d'affichage. */
export type Couleur = "rouge" | "bleu";

/** Unités connues. */
export enum Unite {
  Metre,
  Pied,
}
