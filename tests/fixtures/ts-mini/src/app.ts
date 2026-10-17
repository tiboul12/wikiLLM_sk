// Point d'entrée de la fixture.

import { doubler, tripler as fois3 } from "./lib/calcul";
import * as calcul from "./lib/calcul";
import { Composant } from "./composant";

/** Calcule le total ; utilise les deux fonctions de calcul. */
export function total(x: number): number {
  const c = new Composant();
  c.afficher();
  return doubler(x) + fois3(x) + calcul.doubler(1);
}
