<script lang="ts">
  // Page fichier (EF-041) : en-tête //! puis une cellule par symbole.
  import { tick } from "svelte";
  import { api } from "../lib/api";
  import { rendre } from "../lib/markdown";
  import type { PageFichier } from "../types/PageFichier";
  import Cellule from "../composants/Cellule.svelte";
  import Balises from "../composants/Balises.svelte";

  let { chemin, ancre }: { chemin: string; ancre: string } = $props();
  let page = $state<PageFichier | null>(null);
  let erreur = $state("");
  let montrerTests = $state(false);

  // Page recréée par {#key} à chaque changement de route : la valeur initiale suffit.
  // svelte-ignore state_referenced_locally
  api
    .fichier(chemin)
    .then(async (p) => {
      page = p;
      await tick();
      if (ancre) document.getElementById(ancre)?.scrollIntoView({ block: "start" });
    })
    .catch((e) => (erreur = String(e)));

  const cellules = $derived(page?.cellules.filter((c) => montrerTests || !c.est_test) ?? []);
  const tests = $derived(page?.cellules.filter((c) => c.est_test).length ?? 0);
  const perimes = $derived(page?.cellules.filter((c) => c.statut === "périmé").length ?? 0);
</script>

{#if erreur}
  <p class="erreur">{erreur}</p>
{:else if page}
  <p class="fil">{chemin.split("/").slice(0, -1).join(" / ")}</p>
  <h1>{chemin.split("/").pop()}</h1>
  <p class="meta">
    {page.lignes} lignes · {page.cellules.length - tests} symboles{#if perimes}<span class="statut perime">{perimes} doc périmée(s)</span>{/if}
    {#if page.lignes_macros}· {Math.round((100 * page.lignes_macros) / Math.max(page.lignes, 1))} % en macros (appels non analysés){/if}
  </p>
  {#if page.entete || page.balises_entete.length}
    <div class="entete-fichier">
      {#if page.entete}<div class="prose">{@html rendre(page.entete)}</div>{/if}
      <Balises balises={page.balises_entete} />
    </div>
  {:else}
    <p class="discret">Pas de bloc d'en-tête <code>//!</code>.</p>
  {/if}
  {#each cellules as c (c.id)}
    <Cellule cellule={c} langage={page.langage} ouverte={ancre === c.nom_qualifie && !c.doc} />
  {/each}
  {#if tests}
    <button class="secondaire" onclick={() => (montrerTests = !montrerTests)}>
      {montrerTests ? "Masquer" : "Afficher"} les {tests} symbole(s) de test
    </button>
  {/if}
{:else}
  <p class="discret">Chargement…</p>
{/if}
