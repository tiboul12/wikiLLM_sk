<script lang="ts">
  // Barre d'activité, à la VS Code (EF-131) : une icône par section, toujours visible, avec un badge pour
  // ce qui attend (remarques ouvertes en Réception, clarifications ouvertes en Projet) ; le graphe en bas. Sur téléphone, elle
  // passe en barre du bas.
  import { SECTIONS, NOMS, type Section } from "../lib/sections.svelte";
  import { url } from "../lib/routeur.svelte";
  import { atelier } from "../lib/atelier.svelte";
  import { reception, actuelles } from "../lib/reception.svelte";
  import Icone from "./Icone.svelte";

  let { active, graphe = false }: { active: Section | null; graphe?: boolean } = $props();
  const LIENS: Record<Section, () => string> = {
    accueil: url.accueil,
    produit: url.produit,
    projet: url.projet,
    atelier: () => url.atelier(atelier.parcours),
    reception: url.reception,
  };
  const badge = (s: Section): number => {
    if (s === "reception") return reception.etat ? actuelles().filter((r) => r.statut === "ouverte").length : 0;
    if (s === "projet") return atelier.etat?.clarifications.filter((c) => c.frontmatter.etat === "ouverte").length ?? 0;
    return 0;
  };
</script>

<nav class="barre-activite" aria-label="Sections">
  {#each SECTIONS as s (s)}
    {@const n = badge(s)}
    <a href={LIENS[s]()} class:actif={active === s} title={NOMS[s]} aria-current={active === s ? "page" : undefined}>
      <Icone nom={s} />
      <span class="libelle">{NOMS[s]}</span>
      {#if n}<span class="badge" title="{n} en attente">{n}</span>{/if}
    </a>
  {/each}
  <span class="espace"></span>
  <a href={url.graphe()} class:actif={graphe} title="Graphe"><Icone nom="graphe" /><span class="libelle">Graphe</span></a>
</nav>
