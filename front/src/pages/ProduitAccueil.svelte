<script lang="ts">
  // Accueil de la section Code, ex-Produit (EF-142, EF-150) : chiffres du code, puis le récapitulatif fichier par fichier
  // (mêmes chiffres que les tuiles, plus ce que `wikik check` y signale), triable et filtrable ; enfin la
  // doc à reprendre.
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

  type Ligne = { chemin: string; lignes: number; symboles: number; documentes: number; taux: number; sansDoc: number; perimes: number; alertes: number };
  const COLONNES: [keyof Ligne, string][] = [
    ["chemin", "Fichier"],
    ["lignes", "Lignes"],
    ["symboles", "Symboles"],
    ["taux", "Documentés"],
    ["sansDoc", "Sans doc"],
    ["perimes", "Périmés"],
    ["alertes", "Autres alertes"],
  ];
  let tri = $state<keyof Ligne>("chemin");
  let sens = $state(1);
  let filtre = $state("");
  const lignes = $derived.by((): Ligne[] => {
    // « Sans doc » vient de l'index (comme la tuile « documentés ») ; périmés et autres alertes, du check.
    const diag = new Map<string, { perimes: number; alertes: number }>();
    for (const d of rapport?.diagnostics ?? []) {
      if (d.severite === "info" || d.code === "manquant") continue;
      const c = diag.get(d.fichier) ?? { perimes: 0, alertes: 0 };
      if (d.code.startsWith("perime")) c.perimes++;
      else c.alertes++;
      diag.set(d.fichier, c);
    }
    const f = filtre.trim().toLowerCase();
    return (projet?.fichiers ?? [])
      .filter((x) => !f || x.chemin.toLowerCase().includes(f))
      .map((x) => ({
        chemin: x.chemin,
        lignes: x.lignes,
        symboles: x.symboles,
        documentes: x.documentes,
        taux: x.symboles ? Math.round((100 * x.documentes) / x.symboles) : 100,
        sansDoc: x.symboles - x.documentes,
        perimes: diag.get(x.chemin)?.perimes ?? 0,
        alertes: diag.get(x.chemin)?.alertes ?? 0,
      }))
      .sort((a, b) => (a[tri] < b[tri] ? -sens : a[tri] > b[tri] ? sens : 0));
  });
  function trier(c: keyof Ligne) {
    if (tri === c) sens = -sens;
    else {
      tri = c;
      // Les colonnes de chiffres se lisent du pire au meilleur.
      sens = c === "chemin" || c === "taux" ? 1 : -1;
    }
  }
</script>

{#if projet}
  <h1>Code</h1>
  <p class="discret">Le code et sa doc : ouvre un fichier dans la barre de gauche ou dans le tableau ; chaque symbole y a sa cellule. Les liens entre tout le projet : <a href={url.graphe()}>le graphe</a>.</p>
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

  <h2>Fichier par fichier</h2>
  <p class="filtre-tableau"><input type="search" bind:value={filtre} placeholder="Filtrer par chemin…" aria-label="Filtrer les fichiers" /> <span class="discret">{lignes.length} fichier(s) · clic sur un titre de colonne pour trier</span></p>
  <div class="defile-tableau">
    <table class="recap-fichiers">
      <thead>
        <tr>
          {#each COLONNES as [c, nom] (c)}
            <th class:nombre={c !== "chemin"} aria-sort={tri === c ? (sens > 0 ? "ascending" : "descending") : "none"}>
              <button onclick={() => trier(c)}>{nom}{tri === c ? (sens > 0 ? " ▲" : " ▼") : ""}</button>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each lignes as l (l.chemin)}
          <tr>
            <td><a href={url.fichier(l.chemin)}>{l.chemin}</a></td>
            <td class="nombre">{l.lignes}</td>
            <td class="nombre">{l.symboles}</td>
            <td class="nombre"><span class="taux" class:faible={l.taux < 50}>{l.symboles ? `${l.taux} %` : "—"}</span></td>
            <td class="nombre" class:a-voir={l.sansDoc > 0}>{l.sansDoc || ""}</td>
            <td class="nombre" class:a-voir={l.perimes > 0}>{l.perimes || ""}</td>
            <td class="nombre" class:a-voir={l.alertes > 0}>{l.alertes || ""}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

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
  <h1>Code</h1>
  <p class="discret">Chargement…</p>
{/if}
