<script lang="ts">
  // Une remarque : genre, gravité, texte, captures et historique des versions ; la modifier (nouvelle
  // version), la supprimer (toute la chaîne, après confirmation) ; en traitement, statut et explication.
  import type { Remarque } from "../../types/Remarque";
  import { ecrireReception, GENRES, GRAVITES, STATUTS } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";
  import NouvelleRemarque from "./NouvelleRemarque.svelte";

  let { r, traitement = false, avecPage = false }: { r: Remarque; traitement?: boolean; avecPage?: boolean } = $props();
  // svelte-ignore state_referenced_locally
  let resolution = $state(r.resolution ?? "");
  // svelte-ignore state_referenced_locally
  let commit = $state(r.commit ?? "");
  let grande = $state<string | null>(null);
  let erreur = $state("");
  let edition = $state(false);
  let confirmer = $state(false);

  async function traiter(statut: string) {
    erreur = "";
    try {
      await ecrireReception("traiter", { id: r.id, statut, resolution, commit: commit || null });
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }

  async function supprimer() {
    erreur = "";
    try {
      await ecrireReception("supprimer", { id: r.id });
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
      confirmer = false;
    }
  }
  const image = (chemin: string) => `/api/image?chemin=${encodeURIComponent(chemin)}`;
  const versions = $derived(r.historique.length ? r.historique : [r.id]);
</script>

{#if edition}
  <NouvelleRemarque page={r.page} remarque={r} fini={() => (edition = false)} />
{:else}
  <article class="remarque gravite-{r.gravite} statut-{r.statut}">
    <header>
      <a class="id" href={url.document(r.id)}>{r.id}</a>
      <span class="pastille genre-{r.genre}">{GENRES[r.genre] ?? r.genre}</span>
      <span class="pastille gravite">{GRAVITES[r.gravite] ?? r.gravite}</span>
      {#if r.statut !== "ouverte"}<span class="pastille statut">{STATUTS[r.statut] ?? r.statut}</span>{/if}
      {#if avecPage}<a class="page-de" href={url.pageReception(r.page)}>{r.nom_page}</a>{/if}
      <span class="outils-remarque">
        <span class="discret date">{r.date}</span>
        <button class="discret-bouton" onclick={() => (edition = true)} title="Modifier : crée une nouvelle version">✎ Modifier</button>
        <button class="discret-bouton danger" onclick={() => (confirmer = true)} title="Supprimer la remarque et son historique">✕ Supprimer</button>
      </span>
    </header>
    {#if r.texte}<p class="texte-libre">{r.texte}</p>{/if}
    {#if r.images.length}
      <div class="captures">
        {#each r.images as i}
          <button class="capture" onclick={() => (grande = i)} title="Agrandir"><img src={image(i)} alt="capture" loading="lazy" /></button>
        {/each}
      </div>
    {/if}
    {#if r.historique.length > 1}
      <p class="historique">
        <span class="discret">Historique :</span>
        {#each r.historique as v, n}{#if n}<span class="discret">{" → "}</span>{/if}<a class:courante={v === r.id} href={url.document(v)}>{v}</a>{/each}
      </p>
    {/if}
    {#if confirmer}
      <div class="confirmation" role="alertdialog">
        Supprimer {versions.join(", ")} et {versions.length > 1 ? "leurs" : "ses"} captures ? (Le dépôt git en garde la trace.)
        <div class="actions">
          <button class="principal danger" onclick={supprimer}>Supprimer</button>
          <button onclick={() => (confirmer = false)}>Annuler</button>
        </div>
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
      </div>
    {/if}
    {#if erreur}<p class="erreur">{erreur}</p>{/if}
  </article>
{/if}
{#if grande}
  <button class="visionneuse" onclick={() => (grande = null)} aria-label="Fermer"><img src={image(grande)} alt="capture agrandie" /></button>
{/if}
