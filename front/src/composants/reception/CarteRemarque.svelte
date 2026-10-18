<script lang="ts">
  // Une remarque : genre, gravité, texte et captures ; en traitement, le statut et son explication.
  import type { Remarque } from "../../types/Remarque";
  import { ecrireReception, GENRES, GRAVITES, STATUTS } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";

  let { r, traitement = false, avecPage = false }: { r: Remarque; traitement?: boolean; avecPage?: boolean } = $props();
  // svelte-ignore state_referenced_locally
  let resolution = $state(r.resolution ?? "");
  // svelte-ignore state_referenced_locally
  let commit = $state(r.commit ?? "");
  let grande = $state<string | null>(null);
  let erreur = $state("");

  async function traiter(statut: string) {
    erreur = "";
    try {
      await ecrireReception("traiter", { id: r.id, statut, resolution, commit: commit || null });
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }
  const image = (chemin: string) => `/api/image?chemin=${encodeURIComponent(chemin)}`;
</script>

<article class="remarque gravite-{r.gravite} statut-{r.statut}">
  <header>
    <a class="id" href={url.document(r.id)}>{r.id}</a>
    <span class="pastille genre-{r.genre}">{GENRES[r.genre] ?? r.genre}</span>
    <span class="pastille gravite">{GRAVITES[r.gravite] ?? r.gravite}</span>
    {#if r.statut !== "ouverte"}<span class="pastille statut">{STATUTS[r.statut] ?? r.statut}</span>{/if}
    {#if avecPage}<a class="page-de" href={url.pageReception(r.page)}>{r.nom_page}</a>{/if}
    <span class="discret date">{r.date}</span>
  </header>
  {#if r.texte}<p class="texte-libre">{r.texte}</p>{/if}
  {#if r.images.length}
    <div class="captures">
      {#each r.images as i}
        <button class="capture" onclick={() => (grande = i)} title="Agrandir"><img src={image(i)} alt="capture" loading="lazy" /></button>
      {/each}
    </div>
  {/if}
  {#if r.resolution && !traitement}<p class="resolution"><strong>{STATUTS[r.statut]}</strong> : {r.resolution}{r.commit ? ` (${r.commit})` : ""}</p>{/if}
  {#if traitement}
    <div class="traiter">
      <input bind:value={resolution} placeholder="Ce qui a été fait, ou pourquoi c'est reporté / rejeté" aria-label="Explication" />
      <input class="commit" bind:value={commit} placeholder="commit" aria-label="Commit" />
      <div class="actions">
        {#each ["traitee", "reportee", "rejetee"] as s}
          <button class:principal={s === "traitee"} class:actif={r.statut === s} onclick={() => traiter(s)} disabled={!resolution.trim()}>{STATUTS[s]}</button>
        {/each}
        {#if r.statut !== "ouverte"}<button class="discret-bouton" onclick={() => traiter("ouverte")}>Rouvrir</button>{/if}
      </div>
      {#if erreur}<p class="erreur">{erreur}</p>{/if}
    </div>
  {/if}
</article>
{#if grande}
  <button class="visionneuse" onclick={() => (grande = null)} aria-label="Fermer"><img src={image(grande)} alt="capture agrandie" /></button>
{/if}
