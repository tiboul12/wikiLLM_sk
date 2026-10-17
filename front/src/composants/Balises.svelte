<script lang="ts">
  // Balises d'un bloc en pastilles : @adr et @req mènent au document, @pitfall est mis en avant.
  import type { BaliseVue } from "../types/BaliseVue";
  import Lien from "./Lien.svelte";

  let { balises }: { balises: BaliseVue[] } = $props();
</script>

{#if balises.length}
  <ul class="balises">
    {#each balises as b}
      <li class="balise b-{b.nom}">
        <span class="nom">@{b.nom}</span>
        {#if b.cibles.length}
          {#each b.cibles as c}<Lien lien={c} />{/each}
        {:else}
          {b.valeur}
        {/if}
      </li>
    {/each}
  </ul>
{/if}
