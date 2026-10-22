<script lang="ts">
  // Planche d'un storyboard : personnage, scène, intrigue, puis les vignettes et les artefacts à produire.
  // Chaque vignette se dessine dans l'éditeur vectoriel, et la planche reçoit des maquettes d'interface
  // (EF-167, REC-078).
  import { artefact, champ } from "../../lib/atelier.svelte";
  import { terminal } from "../../lib/terminal.svelte";
  import DessinSvg from "./DessinSvg.svelte";
  import ApercuDessin from "./ApercuDessin.svelte";

  let { id }: { id: string } = $props();
  const sb = $derived(artefact(id));
  type Vignette = { id: string; titre: string; description: string; prompt: string };
  type Art = { id: string; nom: string; genre: string; vignette: string; description: string };
  const vignettes = $derived((sb?.frontmatter.vignettes ?? []) as Vignette[]);
  const arts = $derived((sb?.frontmatter.artefacts ?? []) as Art[]);

  let dessins = $state<string[]>([]);
  let version = $state(0);
  const charger = () => fetch("/api/dessins").then((r) => r.json()).then((d) => (dessins = d)).catch(() => {});
  charger();
  let ouvert = $state<{ nom: string; titre: string } | null>(null);
  const maquettes = $derived(dessins.filter((d) => d.startsWith(`${id}-maquette-`)));
  function nouvelleMaquette() {
    const n = Math.max(0, ...maquettes.map((m) => Number(m.split("-").pop()) || 0)) + 1;
    ouvert = { nom: `${id}-maquette-${n}`, titre: `${id} · maquette ${n}` };
  }
  function ferme(enregistre: boolean) {
    ouvert = null;
    if (enregistre) {
      version++;
      charger();
    }
  }
</script>

{#if ouvert}<DessinSvg nom={ouvert.nom} titre={ouvert.titre} fermer={ferme} />{/if}
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
          {@const nomDessin = `${id}-${v.id}`}
          <li class="vignette" class:avec-dessin={dessins.includes(nomDessin)}>
            <strong>{v.id} · {v.titre}</strong>
            {#if dessins.includes(nomDessin)}<ApercuDessin nom={nomDessin} {version} />{/if}
            <p>{v.description}</p>
            <p class="prompt"><span class="discret">prompt :</span> {v.prompt}</p>
            {#if terminal.proprietaire}<button class="discret-bouton" onclick={() => (ouvert = { nom: nomDessin, titre: `${id} · ${v.id} · ${v.titre}` })}>✎ {dessins.includes(nomDessin) ? "Modifier le dessin" : "Dessiner"}</button>{/if}
          </li>
        {/each}
      </ol>
    {/if}
    <div class="maquettes">
      <strong>Maquettes d'interface</strong>
      {#each maquettes as m (m)}
        <div class="maquette"><ApercuDessin nom={m} {version} />{#if terminal.proprietaire}<button class="discret-bouton" onclick={() => (ouvert = { nom: m, titre: m })}>✎ Modifier</button>{/if}</div>
      {/each}
      {#if terminal.proprietaire}<button class="discret-bouton" onclick={nouvelleMaquette}>＋ Nouvelle maquette</button>{/if}
    </div>
    {#if arts.length}
      <ul class="arts">{#each arts as a}<li><span class="id">{a.id}</span> {a.nom} <span class="pastille">{a.genre}</span> <span class="discret">({a.vignette})</span></li>{/each}</ul>
    {/if}
  </div>
{/if}

<style>
  .vignette.avec-dessin {
    aspect-ratio: auto;
  }
  .maquettes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 10px;
    align-items: start;
    margin: 10px 0;
  }
  .maquettes > strong {
    grid-column: 1 / -1;
  }
</style>
