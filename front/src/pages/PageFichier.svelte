<script lang="ts">
  // Page fichier (EF-041, EF-142) : récapitulatif de ce qui n'est pas conforme (sans doc, doc périmée,
  // sans appelant, alertes du check), en-tête //! puis une cellule par symbole.
  // @ecrans /produit/:chemin Page d'un fichier de code
  import { tick } from "svelte";
  import { api, type RapportCheck } from "../lib/api";
  import { url } from "../lib/routeur.svelte";
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

  let rapport = $state<RapportCheck | null>(null);
  api.check().then((r) => (rapport = r)).catch(() => {});

  // Un lien vers un autre symbole du même fichier change seulement l'ancre : on y va sans recharger.
  $effect(() => {
    if (page && ancre) tick().then(() => document.getElementById(ancre)?.scrollIntoView({ block: "start" }));
  });

  const vrais = $derived(page?.cellules.filter((c) => !c.est_test) ?? []);
  const sansDoc = $derived(vrais.filter((c) => c.statut === "sans doc"));
  const aPerime = $derived(vrais.filter((c) => c.statut === "périmé"));
  // « Pas utilisé » : fonction privée sans appelant connu (une fonction publique peut servir ailleurs).
  const inutilises = $derived(vrais.filter((c) => c.visibilite === "prive" && /fn|fonction|methode|méthode/.test(c.genre) && !c.appelants.length && c.nom_qualifie !== "main"));
  const alertes = $derived(
    rapport?.diagnostics.filter((d) => d.fichier === chemin && d.severite !== "info" && d.code !== "manquant" && !d.code.startsWith("perime")) ?? [],
  );
  const conforme = $derived(!sansDoc.length && !aPerime.length && !inutilises.length && !alertes.length);
  const lienSymbole = (nq: string) => url.fichier(chemin, nq);

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
  <section class="recap-conformite" class:ok={conforme} aria-label="Conformité du fichier">
    {#if conforme}
      ✓ Conforme : chaque symbole est documenté, aucune doc périmée, aucune alerte.
    {:else}
      <strong>À reprendre dans ce fichier</strong>
      <ul>
        {#if sansDoc.length}<li>Sans doc ({sansDoc.length}) : <span class="noms">{#each sansDoc as c (c.id)}<a href={lienSymbole(c.nom_qualifie)}>{c.nom_qualifie}</a>{/each}</span></li>{/if}
        {#if aPerime.length}<li>Doc périmée ({aPerime.length}) : <span class="noms">{#each aPerime as c (c.id)}<a href={lienSymbole(c.nom_qualifie)}>{c.nom_qualifie}</a>{/each}</span></li>{/if}
        {#if inutilises.length}<li>Sans appelant connu, peut-être inutilisé ({inutilises.length}) : <span class="noms">{#each inutilises as c (c.id)}<a href={lienSymbole(c.nom_qualifie)}>{c.nom_qualifie}</a>{/each}</span></li>{/if}
        {#each alertes as d}<li><code>{d.code}</code> ligne {d.ligne} : {d.message}</li>{/each}
      </ul>
    {/if}
  </section>
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
