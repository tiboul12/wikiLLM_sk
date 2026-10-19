// Sections de l'interface (ADR-0012, EF-131) : Accueil, Projet, Design thinking (l'atelier), Produit,
// Réception, dans l'ordre où l'on s'en sert (EF-140). La section active se déduit de la page affichée ; une
// page commune (recherche, graphe) garde la section d'où l'on vient.
//
// @req EF-131, EF-140
import type { Route } from "./routeur.svelte";
import type { Projet } from "../types/Projet";
import { statique } from "./statique";

/** Une section de l'interface (une icône de la barre d'activité). */
export type Section = "accueil" | "produit" | "projet" | "atelier" | "reception";

/** Sections présentes : le site exporté n'a ni atelier ni réception.
 * @relu 2026-10-19 5f1194ed
 */
export const SECTIONS: Section[] = statique ? ["accueil", "projet", "produit"] : ["accueil", "projet", "atelier", "produit", "reception"];

/** Nom affiché de chaque section.
 * @relu 2026-10-19 6c2ea740
 */
export const NOMS: Record<Section, string> = {
  accueil: "Accueil",
  produit: "Produit",
  projet: "Projet",
  atelier: "Design thinking",
  reception: "Réception",
};

/** Section d'une page ; `null` pour une page commune à toutes (recherche, graphe).
 * @relu 2026-10-19 9ef9eb13
 */
export function sectionDe(route: Route, projet: Projet | null): Section | null {
  switch (route.page) {
    case "accueil":
      return "accueil";
    case "produit":
    case "fichier":
      return "produit";
    case "projet":
    case "relecture":
      return "projet";
    case "document": {
      const zone = projet?.documents.find((d) => d.id === route.id)?.zone;
      return zone === "atelier" ? "atelier" : zone === "reception" ? "reception" : "projet";
    }
    case "atelier":
    case "etape":
    case "clarification":
    case "clarifier":
      return "atelier";
    case "reception":
    case "reception-page":
    case "traitement":
      return "reception";
    default:
      return null;
  }
}

/** Dernière section ouverte, gardée pour les pages communes. */
export const navigation = $state({ section: "accueil" as Section });
