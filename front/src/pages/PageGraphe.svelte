<script lang="ts">
  // Graphe (EF-043, EF-141) : graphe local d'un symbole ou d'un document, ou graphe global (documents,
  // code ramené au fichier, pages de l'application, remarques). Une couleur et une puce par catégorie ;
  // même dessin d'une ouverture à l'autre ; titre au survol ; appui long : les liens du nœud en couleur ; un
  // clic ouvre le nœud dans un nouvel onglet (EF-151).
  import { onDestroy } from "svelte";
  import cytoscape from "cytoscape";
  import { api } from "../lib/api";
  import { url } from "../lib/routeur.svelte";
  import { ouvrir } from "../lib/onglets.svelte";
  import { statique } from "../lib/statique";
  import type { Graphe } from "../types/Graphe";

  let { centre }: { centre: string } = $props();
  let conteneur = $state<HTMLElement | null>(null);
  let graphe = $state<Graphe | null>(null);
  let profondeur = $state(1);
  let cy: cytoscape.Core | null = null;

  /** Catégories, dans l'ordre des puces : nom affiché et variable de couleur. */
  const CATEGORIES: [string, string, string][] = [
    ["code", "Code", "--accent"],
    ["exigences", "Besoins et exigences", "--vert"],
    ["decisions", "Décisions", "--violet"],
    ["projet", "Documents de projet", "--gris"],
    ["atelier", "Avant-projet (atelier)", "--orange"],
    ["pages", "Pages de l'application", "--cyan"],
    ["reception", "Réception", "--rouge"],
    ["absent", "Introuvable", "--bord-fort"],
  ];
  let masquees = $state(new Set<string>());
  let survol = $state<{ x: number; y: number; id: string; titre: string; categorie: string } | null>(null);
  let focus = $state<{ id: string; liens: number } | null>(null);

  $effect(() => {
    api.graphe(centre || undefined, profondeur).then((g) => (graphe = g));
  });

  const couleur = (nom: string) => getComputedStyle(document.documentElement).getPropertyValue(nom).trim();
  const couleurDe = (cat: string) => couleur(CATEGORIES.find((c) => c[0] === cat)?.[2] ?? "--accent");
  const presentes = $derived(CATEGORIES.filter(([c]) => graphe?.noeuds.some((n) => n.categorie === c)));
  const compte = (cat: string) => graphe?.noeuds.filter((n) => n.categorie === cat).length ?? 0;

  /** Générateur pseudo-aléatoire à graine fixe : la disposition « cose » est la même à chaque ouverture (REC-006). */
  function graine(n: number) {
    return () => {
      n = (n + 0x6d2b79f5) | 0;
      let t = Math.imul(n ^ (n >>> 15), 1 | n);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }
  /** Position de départ tirée de l'identifiant : même graphe, même dessin. */
  function depart(id: string, i: number, n: number) {
    let h = 0;
    for (const c of id) h = (Math.imul(h, 31) + c.charCodeAt(0)) | 0;
    const a = (2 * Math.PI * i) / Math.max(n, 1);
    const r = 300 + (Math.abs(h) % 200);
    return { x: r * Math.cos(a), y: r * Math.sin(a) };
  }

  $effect(() => {
    if (!conteneur || !graphe) return;
    cy?.destroy();
    focus = null;
    const n = graphe.noeuds.length;
    const ordonnes = [...graphe.noeuds].sort((a, b) => a.id.localeCompare(b.id));
    cy = cytoscape({
      container: conteneur,
      elements: [
        ...ordonnes.map((x, i) => ({
          data: { id: x.id, label: x.libelle, genre: x.genre, categorie: x.categorie, titre: x.titre, fichier: x.fichier ?? "", centre: x.centre ? 1 : 0, couleur: couleurDe(x.categorie) },
          position: depart(x.id, i, n),
        })),
        ...graphe.aretes.map((a, i) => ({ data: { id: `a${i}`, source: a.source, target: a.cible, genre: a.genre, confiance: a.confiance } })),
      ],
      style: [
        { selector: "node", style: { label: "data(label)", "font-size": 9, color: couleur("--texte"), "background-color": "data(couleur)", width: 12, height: 12, "text-valign": "bottom", "text-margin-y": 3, "min-zoomed-font-size": 5 } },
        { selector: "node[genre = 'document']", style: { shape: "round-rectangle" } },
        { selector: "node[genre = 'page']", style: { shape: "hexagon", width: 15, height: 15 } },
        { selector: "node[genre = 'topic']", style: { shape: "diamond" } },
        { selector: "node[centre = 1]", style: { width: 22, height: 22, "border-width": 3, "border-color": couleur("--texte") } },
        { selector: "edge", style: { width: 1, "line-color": couleur("--bord-fort"), "target-arrow-color": couleur("--bord-fort"), "target-arrow-shape": "triangle", "curve-style": "bezier", "arrow-scale": 0.7, opacity: 0.7 } },
        { selector: "edge[confiance = 'probable']", style: { "line-style": "dashed" } },
        { selector: ".estompe", style: { opacity: 0.08 } },
        { selector: "edge.lie", style: { width: 2.4, opacity: 1, "line-color": "data(teinte)", "target-arrow-color": "data(teinte)", "z-index": 9 } },
        { selector: "node.lie", style: { "font-size": 11, "font-weight": "bold", "z-index": 10 } },
      ],
      layout: { name: "preset" },
      wheelSensitivity: 0.3,
      maxZoom: 5,
      minZoom: 0.05,
    });
    // Disposition à graine fixe : `Math.random` est remplacé le temps du calcul (synchrone, sans animation).
    const hasard = Math.random;
    Math.random = graine(n * 7919 + graphe.aretes.length);
    try {
      cy.layout({
        name: "cose",
        animate: false,
        randomize: false,
        nodeRepulsion: () => 26000,
        idealEdgeLength: () => 110,
        nodeOverlap: 40,
        componentSpacing: 140,
        gravity: 0.6,
        numIter: 1500,
      } as cytoscape.LayoutOptions).run();
    } finally {
      Math.random = hasard;
    }
    appliquerMasque();

    let tenu = false;
    cy.on("tap", "node", (e) => {
      if (tenu) return void (tenu = false);
      const d = e.target.data();
      // Un nœud s'ouvre dans un nouvel onglet : le graphe reste à portée (REC-068).
      if (d.genre === "symbole") ouvrir(url.fichier(d.fichier, d.id.slice(d.fichier.length + 2)));
      else if (d.genre === "fichier") ouvrir(url.fichier(d.id));
      else if (d.genre === "document") ouvrir(url.document(d.id));
      else if (d.genre === "page" && !statique) ouvrir(url.pageReception(d.id.slice("page:".length)));
    });
    cy.on("taphold", "node", (e) => {
      tenu = true;
      eclairer(e.target);
    });
    cy.on("tap", (e) => {
      if (e.target === cy) effacer();
    });
    cy.on("mouseover", "node", (e) => {
      const d = e.target.data();
      const p = e.renderedPosition ?? e.target.renderedPosition();
      survol = { x: p.x, y: p.y, id: d.genre === "document" ? d.id : d.label, titre: d.titre, categorie: d.categorie };
    });
    cy.on("mouseout", "node", () => (survol = null));
    cy.on("pan zoom", () => (survol = null));
  });

  /** Appui long (REC-003) : les liens du nœud prennent la couleur de l'autre bout, le reste s'estompe. */
  function eclairer(n: cytoscape.NodeSingular) {
    if (!cy) return;
    effacer();
    const aretes = n.connectedEdges();
    cy.batch(() => {
      cy!.elements().addClass("estompe");
      n.removeClass("estompe").addClass("lie");
      aretes.forEach((a) => {
        const autre = a.source().id() === n.id() ? a.target() : a.source();
        a.data("teinte", autre.data("couleur"));
        a.removeClass("estompe").addClass("lie");
        autre.removeClass("estompe").addClass("lie");
      });
    });
    focus = { id: n.data("genre") === "document" ? n.id() : n.data("label"), liens: aretes.length };
  }
  function effacer() {
    cy?.elements().removeClass("estompe lie");
    focus = null;
  }

  function appliquerMasque() {
    cy?.batch(() => cy!.nodes().forEach((x) => void x.style("display", masquees.has(x.data("categorie")) ? "none" : "element")));
  }
  function basculer(cat: string) {
    const m = new Set(masquees);
    if (!m.delete(cat)) m.add(cat);
    masquees = m;
    appliquerMasque();
  }
  const NOMS = Object.fromEntries(CATEGORIES.map(([c, nom]) => [c, nom]));

  onDestroy(() => cy?.destroy());
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && effacer()} />

<h1>{centre ? `Graphe de ${centre.split("::").pop()}` : "Graphe du projet"}</h1>
<div class="barre-graphe">
  {#if centre}
    <label>Profondeur <select bind:value={profondeur}><option value={1}>1</option><option value={2}>2</option></select></label>
  {/if}
  <span class="puces-graphe" role="group" aria-label="Catégories affichées">
    {#each presentes as [c, nom, v] (c)}
      <button class="puce-graphe" class:eteinte={masquees.has(c)} aria-pressed={!masquees.has(c)} onclick={() => basculer(c)} title={masquees.has(c) ? "Afficher" : "Masquer"}>
        <i style="background: var({v})"></i>{nom} <small>{compte(c)}</small>
      </button>
    {/each}
  </span>
  {#if graphe}<span class="discret">{graphe.noeuds.length} nœuds · {graphe.aretes.length} liens · appui long sur un nœud : ses liens</span>{/if}
</div>
{#if focus}
  <p class="focus-graphe">Liens de <strong>{focus.id}</strong> : {focus.liens} · <button class="discret-bouton" onclick={effacer}>effacer (Échap)</button></p>
{/if}
<div class="cadre-graphe">
  <div class="graphe" bind:this={conteneur}></div>
  {#if survol}
    <div class="bulle-graphe" style="left: {survol.x}px; top: {survol.y}px">
      <strong>{survol.id}</strong>
      {#if survol.titre && survol.titre !== survol.id}<span>{survol.titre}</span>{/if}
      <small>{NOMS[survol.categorie] ?? survol.categorie}</small>
    </div>
  {/if}
</div>
