<script lang="ts">
  // Graphe (EF-043) : graphe local d'un symbole ou d'un document, ou graphe de tous les documents.
  import { onDestroy } from "svelte";
  import cytoscape from "cytoscape";
  import { api } from "../lib/api";
  import { aller, url } from "../lib/routeur.svelte";
  import type { Graphe } from "../types/Graphe";

  let { centre }: { centre: string } = $props();
  let conteneur = $state<HTMLElement | null>(null);
  let graphe = $state<Graphe | null>(null);
  let profondeur = $state(1);
  let cy: cytoscape.Core | null = null;

  $effect(() => {
    api.graphe(centre || undefined, profondeur).then((g) => (graphe = g));
  });

  const couleur = (nom: string) => getComputedStyle(document.documentElement).getPropertyValue(nom).trim();

  $effect(() => {
    if (!conteneur || !graphe) return;
    cy?.destroy();
    cy = cytoscape({
      container: conteneur,
      elements: [
        ...graphe.noeuds.map((n) => ({ data: { id: n.id, label: n.libelle, genre: n.genre, fichier: n.fichier ?? "", centre: n.centre ? 1 : 0 } })),
        ...graphe.aretes.map((a, i) => ({ data: { id: `a${i}`, source: a.source, target: a.cible, genre: a.genre, confiance: a.confiance } })),
      ],
      style: [
        { selector: "node", style: { label: "data(label)", "font-size": 10, color: couleur("--texte"), "background-color": couleur("--accent"), width: 14, height: 14, "text-valign": "bottom", "text-margin-y": 4 } },
        { selector: "node[genre = 'document']", style: { "background-color": couleur("--vert"), shape: "round-rectangle" } },
        { selector: "node[genre = 'absent']", style: { "background-color": couleur("--rouge") } },
        { selector: "node[genre = 'topic']", style: { "background-color": couleur("--violet"), shape: "diamond" } },
        { selector: "node[centre = 1]", style: { width: 24, height: 24, "border-width": 3, "border-color": couleur("--texte") } },
        { selector: "edge", style: { width: 1.2, "line-color": couleur("--bord-fort"), "target-arrow-color": couleur("--bord-fort"), "target-arrow-shape": "triangle", "curve-style": "bezier", "arrow-scale": 0.8 } },
        { selector: "edge[confiance = 'probable']", style: { "line-style": "dashed" } },
        { selector: "edge[genre = 'adr'], edge[genre = 'req']", style: { "line-color": couleur("--vert"), "target-arrow-color": couleur("--vert") } },
      ],
      layout: { name: graphe.noeuds.length > 80 ? "cose" : "cose", animate: false, nodeRepulsion: () => 9000, idealEdgeLength: () => 70 } as cytoscape.LayoutOptions,
      wheelSensitivity: 0.3,
      maxZoom: 1.4,
      minZoom: 0.15,
    });
    cy.on("tap", "node", (e) => {
      const d = e.target.data();
      if (d.genre === "symbole") aller(url.fichier(d.fichier, d.id.slice(d.fichier.length + 2)));
      else if (d.genre === "document") aller(url.document(d.id));
    });
  });

  onDestroy(() => cy?.destroy());
</script>

<h1>{centre ? `Graphe de ${centre.split("::").pop()}` : "Graphe des documents"}</h1>
<div class="barre-graphe">
  {#if centre}
    <label>Profondeur <select bind:value={profondeur}><option value={1}>1</option><option value={2}>2</option></select></label>
  {/if}
  <span class="legende"><i class="pt symbole"></i>symbole <i class="pt document"></i>document <i class="pt absent"></i>introuvable <i class="trait"></i>certain <i class="trait pointille"></i>probable</span>
  {#if graphe}<span class="discret">{graphe.noeuds.length} nœuds · {graphe.aretes.length} arêtes</span>{/if}
</div>
<div class="graphe" bind:this={conteneur}></div>
