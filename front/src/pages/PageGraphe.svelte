<script lang="ts">
  // Graphe (EF-043, EF-141) : graphe local d'un symbole ou d'un document, ou graphe global (documents,
  // code ramené au fichier, pages de l'application, remarques). Une couleur et une puce par catégorie ;
  // même dessin d'une ouverture à l'autre ; titre au survol ; appui long : les liens du nœud en couleur ; un
  // clic ouvre le nœud dans un nouvel onglet (EF-151). Le graphe global et sa disposition sont gardés pour
  // la session (↻ les recharge) ; un champ trouve un nœud et zoome dessus (EF-153).
  import { onDestroy } from "svelte";
  import cytoscape from "cytoscape";
  import { api } from "../lib/api";
  import { grapheGlobal, disposer } from "../lib/graphe";
  import { url } from "../lib/routeur.svelte";
  import { ouvrir } from "../lib/onglets.svelte";
  import { statique } from "../lib/statique";
  import type { Graphe } from "../types/Graphe";

  let { centre }: { centre: string } = $props();
  /** Élément où cytoscape dessine le graphe. */
  let conteneur = $state<HTMLElement | null>(null);
  /** Graphe affiché : local autour de `centre`, global sinon. */
  let graphe = $state<Graphe | null>(null);
  /** Profondeur du graphe local, en nombre de liens depuis le centre. */
  let profondeur = $state(1);
  /** Instance cytoscape, détruite et recréée à chaque nouveau graphe (pas réactive : Svelte n'a pas à la suivre). */
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
  /** Catégories masquées par un clic sur leur puce. */
  let masquees = $state(new Set<string>());
  /** Bulle du nœud survolé, en coordonnées d'écran. */
  let survol = $state<{ x: number; y: number; id: string; titre: string; categorie: string } | null>(null);
  /** Nœud éclairé par un appui long (ou trouvé) et son nombre de liens, rappelés sous le graphe. */
  let focus = $state<{ id: string; liens: number } | null>(null);

  /** Graphe en cours de chargement (le global peut prendre quelques secondes au premier calcul). */
  let chargement = $state(false);
  /** Charge le graphe local, ou le graphe global gardé pour la session ; `forcer` (bouton ↻) le recalcule. */
  function charger(forcer = false) {
    chargement = true;
    const p = centre ? api.graphe(centre, profondeur) : grapheGlobal(forcer);
    p.then((g) => (graphe = g)).finally(() => (chargement = false));
  }
  $effect(() => {
    profondeur;
    charger();
  });

  /** Valeur d'une variable CSS du thème, lue au moment du dessin (cytoscape n'accepte pas `var(...)`). */
  const couleur = (nom: string) => getComputedStyle(document.documentElement).getPropertyValue(nom).trim();
  /** Couleur d'une catégorie dans le thème courant. */
  const couleurDe = (cat: string) => couleur(CATEGORIES.find((c) => c[0] === cat)?.[2] ?? "--accent");
  /** Catégories présentes dans le graphe, les seules qui ont une puce. */
  const presentes = $derived(CATEGORIES.filter(([c]) => graphe?.noeuds.some((n) => n.categorie === c)));
  /** Nombre de nœuds d'une catégorie, affiché sur sa puce. */
  const compte = (cat: string) => graphe?.noeuds.filter((n) => n.categorie === cat).length ?? 0;

  $effect(() => {
    if (!conteneur || !graphe) return;
    cy?.destroy();
    focus = null;
    const positions = disposer(graphe);
    const ids = new Set(graphe.noeuds.map((x) => x.id));
    cy = cytoscape({
      container: conteneur,
      elements: [
        ...graphe.noeuds.map((x) => ({
          data: { id: x.id, label: x.libelle, genre: x.genre, categorie: x.categorie, titre: x.titre, fichier: x.fichier ?? "", centre: x.centre ? 1 : 0, couleur: couleurDe(x.categorie) },
          position: positions[x.id] ?? { x: 0, y: 0 },
        })),
        ...graphe.aretes
          .filter((a) => ids.has(a.source) && ids.has(a.cible))
          .map((a, i) => ({ data: { id: `a${i}`, source: a.source, target: a.cible, genre: a.genre, confiance: a.confiance } })),
      ],
      style: [
        { selector: "node", style: { label: "data(label)", "font-size": 9, color: couleur("--texte"), "background-color": "data(couleur)", width: 12, height: 12, "text-valign": "bottom", "text-margin-y": 3, "min-zoomed-font-size": 5 } },
        { selector: "node[genre = 'document']", style: { shape: "round-rectangle" } },
        { selector: "node[genre = 'page']", style: { shape: "hexagon", width: 15, height: 15 } },
        { selector: "node[genre = 'topic']", style: { shape: "diamond" } },
        { selector: "node[centre = 1]", style: { width: 22, height: 22, "border-width": 3, "border-color": couleur("--texte") } },
        { selector: "edge", style: { width: 1, "line-color": couleur("--bord-fort"), "target-arrow-color": couleur("--bord-fort"), "target-arrow-shape": "triangle", "curve-style": "bezier", "arrow-scale": 0.7, opacity: 0.7 } },
        { selector: "edge[confiance = 'probable']", style: { "line-style": "dashed" } },
        { selector: "edge[genre = 'appel'], edge[genre = 'rend']", style: { "line-color": couleurDe("code"), "target-arrow-color": couleurDe("code"), opacity: 0.35 } },
        { selector: "edge[genre = 'navigue'], edge[genre = 'contient']", style: { "line-color": couleurDe("pages"), "target-arrow-color": couleurDe("pages"), opacity: 0.6 } },
        { selector: "edge[genre = 'contient']", style: { "line-style": "dotted" } },
        { selector: ".estompe", style: { opacity: 0.08 } },
        { selector: "edge.lie", style: { width: 2.4, opacity: 1, "line-color": "data(teinte)", "target-arrow-color": "data(teinte)", "z-index": 9 } },
        { selector: "node.lie", style: { "font-size": 11, "font-weight": "bold", "z-index": 10 } },
        { selector: "node.trouve", style: { width: 26, height: 26, "border-width": 4, "border-color": couleur("--accent"), "font-size": 14, "font-weight": "bold", "z-index": 11 } },
      ],
      layout: { name: "preset" },
      wheelSensitivity: 0.3,
      maxZoom: 5,
      minZoom: 0.05,
    });
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
  /** Retire l'éclairage : tout redevient visible. */
  function effacer() {
    cy?.elements().removeClass("estompe lie");
    focus = null;
  }

  /** Texte cherché dans le graphe (REC-070) : le nœud trouvé est mis en avant, centré et zoomé. */
  let quete = $state("");
  /** Aucun nœud visible ne correspond à la recherche. */
  let introuvable = $state(false);
  /** Suggestions du champ : identifiant et titre des documents, libellé des pages, titre des symboles. */
  const suggestions = $derived((graphe?.noeuds ?? []).map((x) => (x.genre === "document" ? `${x.id} · ${x.titre}` : x.genre === "page" ? x.libelle : x.titre || x.libelle)));
  /**
   * Trouve le meilleur nœud visible (égalité exacte, puis fin de chemin, puis contient) et zoome dessus.
   * Une suggestion « ID · titre » est ramenée à son identifiant.
   */
  function trouver(e?: SubmitEvent) {
    e?.preventDefault();
    if (!cy || !quete.trim()) return;
    const q = quete.trim().toLowerCase().split(" · ")[0];
    const score = (x: cytoscape.NodeSingular) => {
      const champs = [x.id(), x.data("label"), x.data("titre")].map((v) => String(v ?? "").toLowerCase());
      if (champs.some((v) => v === q)) return 3;
      if (champs.some((v) => v.endsWith("/" + q) || v.endsWith("::" + q))) return 2;
      return champs.some((v) => v.includes(q)) ? 1 : 0;
    };
    let meilleur: cytoscape.NodeSingular | null = null;
    let s = 0;
    cy.nodes().forEach((x) => {
      const v = x.style("display") === "none" ? 0 : score(x);
      if (v > s) [meilleur, s] = [x, v];
    });
    introuvable = !meilleur;
    if (!meilleur) return;
    const n = meilleur as cytoscape.NodeSingular;
    cy.nodes(".trouve").removeClass("trouve");
    eclairer(n);
    n.addClass("trouve");
    cy.animate({ center: { eles: n }, zoom: 2.4 }, { duration: 450 });
  }

  /** Cache les nœuds des catégories masquées ; leurs liens disparaissent avec eux. */
  function appliquerMasque() {
    cy?.batch(() => cy!.nodes().forEach((x) => void x.style("display", masquees.has(x.data("categorie")) ? "none" : "element")));
  }
  /** Masque ou réaffiche une catégorie. */
  function basculer(cat: string) {
    const m = new Set(masquees);
    if (!m.delete(cat)) m.add(cat);
    masquees = m;
    appliquerMasque();
  }
  /** Nom affiché de chaque catégorie, pour la bulle de survol. */
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
<form class="chercher-graphe" onsubmit={trouver} role="search">
  <input type="search" list="noeuds-graphe" bind:value={quete} placeholder="Trouver un nœud : EF-140, wiki.rs, Clarifications…" aria-label="Trouver un nœud" oninput={() => (introuvable = false)} />
  <datalist id="noeuds-graphe">{#each suggestions as v}<option value={v}></option>{/each}</datalist>
  <button type="submit" disabled={!quete.trim()}>Trouver</button>
  {#if introuvable}<span class="erreur">Aucun nœud ne correspond.</span>{/if}
</form>
{#if focus}
  <p class="focus-graphe">Liens de <strong>{focus.id}</strong> : {focus.liens} · <button class="discret-bouton" onclick={effacer}>effacer (Échap)</button></p>
{/if}
<div class="cadre-graphe">
  <div class="graphe" bind:this={conteneur}></div>
  <button class="rafraichir-graphe" onclick={() => charger(true)} disabled={chargement} title="Recharger le graphe (après des modifications du projet)" aria-label="Recharger le graphe">{chargement ? "…" : "↻"}</button>
  {#if survol}
    <div class="bulle-graphe" style="left: {survol.x}px; top: {survol.y}px">
      <strong>{survol.id}</strong>
      {#if survol.titre && survol.titre !== survol.id}<span>{survol.titre}</span>{/if}
      <small>{NOMS[survol.categorie] ?? survol.categorie}</small>
    </div>
  {/if}
</div>

<style>
  .chercher-graphe {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    margin: 8px 0;
  }
  .chercher-graphe input {
    flex: 1 1 18rem;
    min-width: 0;
    max-width: 32rem;
  }
  .rafraichir-graphe {
    position: absolute;
    top: 10px;
    right: 10px;
    z-index: 2;
    width: 34px;
    height: 34px;
    border: 1px solid var(--bord-fort);
    border-radius: var(--rayon);
    background: var(--fond);
    color: var(--texte);
    font-size: 18px;
    cursor: pointer;
  }
  .rafraichir-graphe:hover {
    background: var(--fond-3);
  }
</style>
