import { total } from "./app";

/** Vérifie le total. */
function testTotal() {
  if (total(1) !== 6) throw new Error("total");
}
