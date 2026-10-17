<script lang="ts">
  // Planche d'un storyboard : personnage, scène, intrigue, puis les vignettes et les artefacts à produire.
  import { artefact, champ } from "../../lib/atelier.svelte";

  let { id }: { id: string } = $props();
  const sb = $derived(artefact(id));
  type Vignette = { id: string; titre: string; description: string; prompt: string };
  type Art = { id: string; nom: string; genre: string; vignette: string; description: string };
  const vignettes = $derived((sb?.frontmatter.vignettes ?? []) as Vignette[]);
  const arts = $derived((sb?.frontmatter.artefacts ?? []) as Art[]);
</script>

{#if sb}
  <div class="planche">
    <p class="scenario">{champ(sb, "scenario")}</p>
    {#if champ(sb, "personnage")}
      <dl class="champs cadre-sb">
        <dt>personnage</dt><dd>{champ(sb, "personnage")}</dd>
        <dt>scène</dt><dd>{champ(sb, "scene")}</dd>
        <dt>intrigue</dt><dd>{champ(sb, "intrigue")}</dd>
      </dl>
    {/if}
    {#if vignettes.length}
      <ol class="vignettes">
        {#each vignettes as v}
          <li class="vignette">
            <strong>{v.id} · {v.titre}</strong>
            <p>{v.description}</p>
            <p class="prompt"><span class="discret">prompt :</span> {v.prompt}</p>
          </li>
        {/each}
      </ol>
    {/if}
    {#if arts.length}
      <ul class="arts">{#each arts as a}<li><span class="id">{a.id}</span> {a.nom} <span class="pastille">{a.genre}</span> <span class="discret">({a.vignette})</span></li>{/each}</ul>
    {/if}
  </div>
{/if}
