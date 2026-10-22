<script lang="ts">
  // Planche d'un storyboard : personnage, scène, intrigue, puis les vignettes et les artefacts à produire.
  // Chaque vignette se dessine dans l'éditeur vectoriel, et la planche reçoit des maquettes d'interface
  // (EF-167, REC-078).
  import { artefact, champ } from "../../lib/atelier.svelte";
  import { terminal } from "../../lib/terminal.svelte";
  import DessinSvg from "./DessinSvg.svelte";
  import ApercuDessin from "./ApercuDessin.svelte";

  let { id }: { id: string } = $props();
  /** Storyboard affiché. */
  const sb = $derived(artefact(id));
  /** Vignette du storyboard telle que le LLM la propose : titre, ce qui se passe, prompt d'illustration. */
  type Vignette = { id: string; titre: string; description: string; prompt: string };
  /** Artefact à produire, rattaché à une vignette (écran, document, objet). */
  type Art = { id: string; nom: string; genre: string; vignette: string; description: string };
  /** Vignettes du storyboard, dans l'ordre du récit. */
  const vignettes = $derived((sb?.frontmatter.vignettes ?? []) as Vignette[]);
  /** Artefacts à produire listés par le storyboard. */
  const arts = $derived((sb?.frontmatter.artefacts ?? []) as Art[]);

  /** Noms des dessins enregistrés, pour savoir quelles vignettes et maquettes existent déjà. */
  let dessins = $state<string[]>([]);
  /** Compteur incrémenté à chaque enregistrement : force le rechargement des aperçus (le navigateur garderait l'ancien SVG). */
  let version = $state(0);
  /** Relit la liste des dessins ; sans serveur (site statique), la liste reste vide. */
  const charger = () => fetch("/api/dessins").then((r) => r.json()).then((d) => (dessins = d)).catch(() => {});
  charger();
  /** Dessin ouvert dans l'éditeur vectoriel : nom du fichier et titre affiché. */
  let ouvert = $state<{ nom: string; titre: string } | null>(null);
  /** Maquettes d'interface de ce storyboard (`<id>-maquette-<n>`). */
  const maquettes = $derived(dessins.filter((d) => d.startsWith(`${id}-maquette-`)));
  /** Ouvre l'éditeur sur une nouvelle maquette, numérotée après la plus haute existante. */
  function nouvelleMaquette() {
    const n = Math.max(0, ...maquettes.map((m) => Number(m.split("-").pop()) || 0)) + 1;
    ouvert = { nom: `${id}-maquette-${n}`, titre: `${id} · maquette ${n}` };
  }
  /** Ferme l'éditeur ; après un enregistrement, relit les dessins et rafraîchit les aperçus. */
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
