<script lang="ts">
  // Page document (EF-042, EF-143) : frontmatter, corps Markdown (images relatives servies par
  // `/api/image`), liens résolus, identifiants et chemins cités liés d'office, ce qui le cite ; liens vers
  // l'atelier pour un besoin, une exigence ou un artefact (SPEC-21). Un plan ou le backlog s'ouvre en
  // tableau à colonnes.
  import { tick } from "svelte";
  import { api } from "../lib/api";
  import { rendre, rendreMermaid } from "../lib/markdown";
  import { brancherHooks, reference } from "../lib/hook";
  import { url } from "../lib/routeur.svelte";
  import type { PageDocument } from "../types/PageDocument";
  import Lien from "../composants/Lien.svelte";
  import { lienAtelier } from "../lib/atelier.svelte";
  import { autoLien } from "../lib/connus.svelte";
  import { colonnesPlan, colonnesBacklog } from "../lib/gestion";
  import Kanban from "../composants/Kanban.svelte";

  let { id, ancre }: { id: string; ancre: string } = $props();
  let page = $state<PageDocument | null>(null);
  let erreur = $state("");
  let corps = $state<HTMLElement | null>(null);

  const CHAMPS = ["statut", "priorite", "version", "date", "besoins", "exigences", "decisions", "remplace", "remplace_par", "sources"];

  // Page recréée par {#key} à chaque changement de route : la valeur initiale suffit.
  // svelte-ignore state_referenced_locally
  api
    .document(id)
    .then((p) => (page = p))
    .catch((e) => (erreur = String(e)));

  const html = $derived.by(() => {
    if (!page) return "";
    const table = new Map(page.liens.map((l) => [l.brute, l.cible]));
    return rendre(
      page.corps,
      (brute) => {
        const c = table.get(brute);
        if (!c) return null;
        return c.fichier ? url.fichier(c.fichier, c.libelle) : url.document(c.id);
      },
      reference(page.id),
      page.chemin,
      autoLien,
    );
  });
  const colonnes = $derived(page?.genre === "plan" ? colonnesPlan(page.corps) : page?.genre === "backlog" ? colonnesBacklog(page.corps) : null);
  let vue = $state<"tableau" | "document">("tableau");

  $effect(() => {
    if (!corps || !html) return;
    brancherHooks(corps);
    tick().then(async () => {
      await rendreMermaid(corps!);
      if (ancre) document.getElementById(ancre)?.scrollIntoView({ block: "start" });
    });
  });

  const valeurs = (v: unknown): string[] => (Array.isArray(v) ? v.map(String) : v === undefined || v === null ? [] : [String(v)]);
  const estId = (v: string) => !v.includes("/") && !v.endsWith(".md");
  // Besoins, exigences et artefacts de l'atelier s'ouvrent dans l'atelier et peuvent y être clarifiés (SPEC-21 R1).
  const RAPPELABLES = ["besoin", "exigence", "exigence-nf"];
  const technique = $derived(typeof page?.frontmatter.technique === "string" ? page.frontmatter.technique : null);
  const ouvrir = $derived(page && (technique || RAPPELABLES.includes(page.genre)) ? lienAtelier(technique && technique !== "clarification" ? `/atelier/${technique}` : page.genre === "clarification" ? `/clarification/${page.id}` : "/atelier") : null);
  const clarifier = $derived(page && page.genre !== "clarification" && (technique || RAPPELABLES.includes(page.genre)) ? lienAtelier(`/clarifier/${encodeURIComponent(page.id)}`) : null);
</script>

{#if erreur}
  <p class="erreur">{erreur}</p>
{:else if page}
  <p class="fil">{page.chemin}</p>
  <div class="frontmatter">
    <span class="genre">{page.genre}</span>
    {#each CHAMPS as champ}
      {#each valeurs(page.frontmatter[champ]) as v, i}
        {#if i === 0}<span class="cle">{champ}</span>{/if}
        {#if estId(v) && ["besoins", "exigences", "decisions", "remplace", "remplace_par", "sources"].includes(champ)}
          <a class="pastille" href={url.document(v)}>{v}</a>
        {:else}
          <span class="pastille {champ === 'statut' ? 'statut-' + v : ''}">{v}</span>
        {/if}
      {/each}
    {/each}
    <a class="action" href={url.graphe(page.id)} title="Graphe local">◎ graphe</a>
    {#if ouvrir}<a class="action atelier" href={ouvrir}>ouvrir dans l'atelier</a>{/if}
    {#if clarifier}<a class="action atelier" href={clarifier}>clarifier</a>{/if}
  </div>
  {#if colonnes}
    <div class="bascule vue-gestion" role="group" aria-label="Affichage">
      <button class:actif={vue === "tableau"} onclick={() => (vue = "tableau")}>Tableau</button>
      <button class:actif={vue === "document"} onclick={() => (vue = "document")}>Document</button>
    </div>
    {#if vue === "tableau"}<Kanban {colonnes} resoudre={autoLien} />{/if}
  {/if}
  <article class="prose document" class:cache={colonnes && vue === "tableau"} bind:this={corps}>{@html html}</article>
  {#if page.cite_par.length || page.references.length}
    <section class="cite-par">
      {#if page.cite_par.length}
        <h2>Cité dans le code</h2>
        <ul>{#each page.cite_par as l}<li><span class="etiquette">@{l.genre}</span> <Lien lien={l} /></li>{/each}</ul>
      {/if}
      {#if page.references.length}
        <h2>Documents qui y renvoient</h2>
        <ul>{#each page.references as l}<li><Lien lien={l} /></li>{/each}</ul>
      {/if}
    </section>
  {/if}
{:else}
  <p class="discret">Chargement…</p>
{/if}
