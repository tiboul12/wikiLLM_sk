<script lang="ts">
  // « En bref » d'une étape (REC-036) : à quoi elle sert, ce que l'on fait, sous-étape par sous-étape, et
  // ce qu'elle laisse dans le projet ; tiré de la fiche, rien n'est écrit à la main.
  import type { Fiche } from "../../types/Fiche";
  import type { SousEtape } from "../../types/SousEtape";
  import { nomArtefact } from "../../lib/atelier.svelte";
  import { rendre } from "../../lib/markdown";

  let { fiche, mode }: { fiche: Fiche; mode: string } = $props();
  /** Valeur du mode choisi (Express ou Complet) d'un réglage de la fiche, ou rien. */
  const m = (x: { express: number; complet: number } | null) => (x ? (mode === "complet" ? x.complet : x.express) : null);

  /** Phrase « ce que l'on fait » d'une sous-étape, tirée de son genre et de ses réglages (durée, variantes, nombre à garder). */
  function phrase(se: SousEtape): string {
    const cible = se.pour_chaque ? ` pour chaque ${nomArtefact(se.pour_chaque, false)} retenu(e)` : "";
    switch (se.genre) {
      case "saisie": {
        const d = m(se.duree_minutes);
        return `Tu écris${se.cree ? ` tes ${nomArtefact(se.cree)}` : ""}${d ? ` (${d} min)` : ""}${se.brouillon ? " ; le LLM peut proposer un brouillon" : ""}.`;
      }
      case "propositions": {
        const v = m(se.variantes);
        const g = m(se.garder);
        const quoi = se.cree ? nomArtefact(se.cree) : se.complete ? `compléments au ${nomArtefact(se.complete, false)}` : "propositions";
        return `Le LLM lit le projet et propose ${v ? `${v} ` : ""}${quoi}${cible} ; tu gardes${g ? ` (${g} conseillé${g > 1 ? "s" : ""})` : ""}, corriges ou rejettes.`;
      }
      case "selection":
        return `Tu choisis les ${nomArtefact(se.parmi)} à approfondir dans la suite.`;
      case "synthese":
        return "Le LLM rédige la synthèse de l'étape à partir de ce que tu as gardé.";
      default:
        return se.genre;
    }
  }
  /** Types d'artefacts que l'étape laisse dans le projet. */
  const produits = $derived([...new Set(fiche.sous_etapes.map((s) => s.cree).filter((x): x is string => !!x))]);
</script>

<section class="en-bref">
  <h2>En bref</h2>
  <div class="prose">{@html rendre(fiche.role, () => null)}</div>
  <ol>
    {#each fiche.sous_etapes as se (se.id)}
      <li><strong>{se.id.replaceAll("_", " ")}</strong> : {phrase(se)}</li>
    {/each}
  </ol>
  {#if produits.length}
    <p class="discret">À la fin : des {produits.map((p) => nomArtefact(p)).join(", ")}, écrits dans <code>docs/atelier/</code> et repris par les étapes suivantes.</p>
  {/if}
</section>
