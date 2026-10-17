<script lang="ts">
  // Recherche plein texte (EF-044) dans les blocs de doc et les documents.
  import { api } from "../lib/api";
  import { url } from "../lib/routeur.svelte";
  import type { Resultat } from "../types/Resultat";

  let { q }: { q: string } = $props();
  let resultats = $state<Resultat[] | null>(null);
  // svelte-ignore state_referenced_locally
  api.cherche(q).then((r) => (resultats = r)).catch(() => (resultats = []));

  const lien = (r: Resultat) => (r.fichier ? url.fichier(r.fichier, r.titre) : url.document(r.cle));
</script>

<h1>Recherche « {q} »</h1>
{#if resultats === null}
  <p class="discret">Recherche…</p>
{:else if resultats.length === 0}
  <p>Aucun résultat.</p>
{:else}
  <ul class="resultats">
    {#each resultats as r}
      <li>
        <span class="genre">{r.genre}</span>
        <a href={lien(r)}>{r.genre === "symbole" ? r.titre : `${r.cle} · ${r.titre}`}</a>
        {#if r.fichier}<span class="discret">{r.fichier}</span>{/if}
        <p>{r.extrait}</p>
      </li>
    {/each}
  </ul>
{/if}
