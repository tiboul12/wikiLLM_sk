<script lang="ts">
  // Documents à relire (EF-142, REC-013) : la liste complète, chacun avec son statut, son type et sa
  // version ; filtres par type et par statut.
  import type { Projet } from "../types/Projet";
  import { url } from "../lib/routeur.svelte";

  let { projet }: { projet: Projet | null } = $props();
  const STATUTS = ["proposé", "brouillon"];
  const docs = $derived((projet?.documents ?? []).filter((d) => d.zone === "projet" && STATUTS.includes(d.statut ?? "")));
  const genres = $derived([...new Set(docs.map((d) => d.genre))].sort());
  let genre = $state("");
  let statut = $state("");
  const version = (chemin: string) => /majeures\/([^/]+)\//.exec(chemin)?.[1] ?? "";
  const liste = $derived(
    docs
      .filter((d) => (!genre || d.genre === genre) && (!statut || d.statut === statut))
      .sort((a, b) => a.genre.localeCompare(b.genre) || a.id.localeCompare(b.id, "fr", { numeric: true })),
  );
</script>

<p class="fil"><a href={url.projet()}>Projet</a> · relecture</p>
<h1>À relire</h1>
{#if !projet}
  <p class="discret">Chargement…</p>
{:else}
  <p class="discret">Les documents au statut « proposé » ou « brouillon » : produits par délégation ou par l'atelier, ils attendent ta relecture. Passe-les à « validé » ou « accepté » dans leur frontmatter une fois relus.</p>
  <div class="actions">
    <label>Type <select bind:value={genre}><option value="">tous ({docs.length})</option>{#each genres as g}<option value={g}>{g} ({docs.filter((d) => d.genre === g).length})</option>{/each}</select></label>
    <label>Statut <select bind:value={statut}><option value="">tous</option>{#each STATUTS as s}<option value={s}>{s}</option>{/each}</select></label>
    <span class="discret">{liste.length} document(s)</span>
  </div>
  <table class="relecture">
    <thead><tr><th>Statut</th><th>Document</th><th>Type</th><th>Version</th></tr></thead>
    <tbody>
      {#each liste as d (d.id)}
        <tr>
          <td><span class="pastille statut-{d.statut}">{d.statut}</span></td>
          <td><a href={url.document(d.id)}><span class="id">{d.id}</span> {d.titre === d.id ? "" : d.titre}</a></td>
          <td class="discret">{d.genre}</td>
          <td class="discret">{version(d.chemin)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
