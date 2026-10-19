<script lang="ts">
  // Accueil de la section Produit : chiffres du code et doc à reprendre (même rapport que `wikik check`).
  import type { Projet } from "../types/Projet";
  import { api, type RapportCheck } from "../lib/api";
  import { url } from "../lib/routeur.svelte";

  let { projet }: { projet: Projet | null } = $props();
  let rapport = $state<RapportCheck | null>(null);
  api.check().then((r) => (rapport = r)).catch(() => {});

  const taux = $derived(projet && projet.symboles ? Math.round((100 * projet.documentes) / projet.symboles) : 0);
  const aReprendre = $derived(
    rapport?.diagnostics.filter((d) => d.severite !== "info").slice(0, 40) ?? [],
  );
  const lienDiag = (d: { fichier: string; symbole: string | null }) =>
    d.fichier.endsWith(".md")
      ? url.document(projet?.documents.find((x) => x.chemin === d.fichier)?.id ?? "")
      : url.fichier(d.fichier, d.symbole?.slice(d.fichier.length + 2));
</script>

{#if projet}
  <h1>Produit</h1>
  <p class="discret">Le code et sa doc : ouvre un fichier dans la barre de gauche ; chaque symbole y a sa cellule.</p>
  <section class="tuiles">
    <div class="tuile"><span class="valeur">{projet.fichiers.length}</span><span class="nom">fichiers de code</span></div>
    <div class="tuile"><span class="valeur">{projet.lignes.toLocaleString("fr")}</span><span class="nom">lignes</span></div>
    <div class="tuile"><span class="valeur">{projet.symboles}</span><span class="nom">symboles</span></div>
    <div class="tuile">
      <span class="valeur">{taux} %</span><span class="nom">documentés</span>
      <span class="jauge"><span style="width: {taux}%"></span></span>
    </div>
    {#if rapport}
      <div class="tuile" class:alerte={rapport.resume.erreurs > 0}>
        <span class="valeur">{rapport.resume.erreurs} · {rapport.resume.avertissements}</span><span class="nom">erreurs · avertissements</span>
      </div>
    {/if}
  </section>

  {#if rapport}
    <h2>À reprendre</h2>
    {#if aReprendre.length === 0}
      <p>Rien à signaler : la doc suit le code.</p>
    {:else}
      <table class="diagnostics">
        <thead><tr><th>Sévérité</th><th>Code</th><th>Où</th><th>Message</th></tr></thead>
        <tbody>
          {#each aReprendre as d}
            <tr class={d.severite}>
              <td>{d.severite}</td>
              <td><code>{d.code}</code></td>
              <td><a href={lienDiag(d)}>{d.fichier}:{d.ligne}</a></td>
              <td>{d.message}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if rapport.diagnostics.length > aReprendre.length}
        <p class="discret">Liste complète : <code>wikik check</code>.</p>
      {/if}
    {/if}
    {#if rapport.mode_reference === "local"}
      <p class="discret">Hors dépôt git : la doc périmée est détectée par une référence locale.</p>
    {/if}
  {/if}
{:else}
  <h1>Produit</h1>
  <p class="discret">Chargement…</p>
{/if}
