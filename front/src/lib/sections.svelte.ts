// Sections de l'interface (ADR-0012, EF-131) : Accueil, Projet, Design thinking (l'atelier), Réception,
// dans l'ordre où l'on s'en sert (EF-140) ; en bas, les outils : Code (l'ancien Produit, avec le graphe,
// EF-150). La section active se déduit de la page affichée ; une page commune (recherche) garde la section
// d'où l'on vient.
//
// @req EF-131, EF-140, EF-150
import type { Route } from "./routeur.svelte";
import type { Projet } from "../types/Projet";
import { statique } from "./statique";

/** Une section de l'interface (une icône de la barre d'activité).
 * @relu 2026-10-19 bcbfecb4
 */
export type Section = "accueil" | "produit" | "projet" | "atelier" | "reception" | "parametres";

/** Sections du haut de la barre : le site exporté n'a ni atelier ni réception. */
export const SECTIONS: Section[] = statique ? ["accueil", "projet"] : ["accueil", "projet", "atelier", "reception"];

/** Outils, en bas de la barre au-dessus des Paramètres (REC-042) : le code et son graphe. */
export const OUTILS: Section[] = ["produit"];

/** Nom affiché de chaque section.
 * @relu 2026-10-22 125cf4c5
 */
export const NOMS: Record<Section, string> = {
  accueil: "Accueil",
  produit: "Code",
  projet: "Projet",
  atelier: "Design thinking",
  reception: "Réception",
  parametres: "Paramètres",
};

/** Section d'une page ; `null` pour une page commune à toutes (recherche) ; le graphe est dans Code.
 * @relu 2026-10-22 fa82f7b1
 */
export function sectionDe(route: Route, projet: Projet | null): Section | null {
  switch (route.page) {
    case "accueil":
      return "accueil";
    case "produit":
    case "fichier":
    case "graphe":
      return "produit";
    case "projet":
    case "relecture":
    case "cascade":
    case "clarification":
    case "clarifier":
    case "clarifications":
      return "projet";
    case "document": {
      const zone = projet?.documents.find((d) => d.id === route.id)?.zone;
      const genre = projet?.documents.find((d) => d.id === route.id)?.genre;
      // Les clarifications vivent dans docs/atelier/ mais se lisent dans Projet (REC-024.2).
      return zone === "atelier" && genre !== "clarification" ? "atelier" : zone === "reception" ? "reception" : "projet";
    }
    case "atelier":
    case "etape":
      return "atelier";
    case "reception":
    case "reception-page":
    case "traitement":
      return "reception";
    case "parametres":
      return "parametres";
    default:
      return null;
  }
}

/** Dernière section ouverte, gardée pour les pages communes. */
export const navigation = $state({ section: "accueil" as Section });
