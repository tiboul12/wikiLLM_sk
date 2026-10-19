<script lang="ts">
  // Tableau à colonnes d'un document de gestion (EF-143) : une colonne par état ou par section, une carte
  // par élément ; les éléments faits sont cochés.
  import type { Colonne } from "../lib/gestion";
  import { rendre } from "../lib/markdown";

  let { colonnes, resoudre }: { colonnes: Colonne[]; resoudre?: (t: string) => string | null } = $props();
  /** Markdown d'une ligne (titre de colonne ou de carte), sans paragraphe autour. */
  const enLigne = (md: string) => rendre(md, resoudre, undefined, undefined, resoudre).replace(/^<p>|<\/p>\s*$/g, "");
</script>

<div class="kanban">
  {#each colonnes as c (c.nom)}
    <section class="colonne-kanban">
      <h3>{@html enLigne(c.nom)} <small>{c.cartes.length}</small></h3>
      {#each c.cartes as k, i (i)}
        <article class="carte-kanban" class:fait={k.fait}>
          <strong>{#if k.fait}✓ {/if}{@html enLigne(k.titre)}</strong>
          {#if k.detail}<div class="prose">{@html rendre(k.detail, resoudre, undefined, undefined, resoudre)}</div>{/if}
          {#if k.date}<small class="discret">{k.date}</small>{/if}
        </article>
      {:else}
        <p class="discret">—</p>
      {/each}
    </section>
  {/each}
</div>
