<script lang="ts">
  // Accueil de la section Projet (EF-131, EF-174, EF-175) : l'état de suivi de chaque version (à relire,
  // validés, tests validés) plutôt qu'un récapitulatif par type (REC-090), puis l'arbre complet des versions
  // et les documents transverses : sur téléphone, sans volet, l'accueil mène à chaque document (REC-099).
  // @req EF-174, EF-175
  import type { Projet } from "../types/Projet";
  import { url } from "../lib/routeur.svelte";
  import { aRelire as aRelireDe } from "../lib/edition.svelte";
  import { rangerParVersion, suivi } from "../lib/versions";
  import ArbreVersions from "../composants/ArbreVersions.svelte";

  let { projet }: { projet: Projet | null } = $props();
  /** Documents qui attendent une relecture. */
  const aRelire = $derived(aRelireDe(projet?.documents ?? []));
  /** Identifiants à relire, pour les compteurs. */
  const ids = $derived(new Set(aRelire.map((d) => d.id)));
  /** Versions et documents transverses. */
  const rangement = $derived(rangerParVersion(projet?.documents ?? []));
  /** Part en pourcentage, 0 sans total. */
  const part = (n: number, total: number) => (total ? Math.round((100 * n) / total) : 0);
</script>

<h1>Projet</h1>
{#if !projet}<p class="discret">Chargement…</p>{:else}
<p class="discret">Les documents du projet, version par version : de la demande d'évolution aux tests. Ils vivent dans <code>docs/projet/</code>.</p>

<div class="tuiles">
  <a class="tuile lien-tuile" class:alerte={aRelire.length > 0} href={url.relecture()}><span class="valeur">{aRelire.length}</span><span class="nom">à relire →</span></a>
  <div class="tuile"><span class="valeur">{rangement.versions.length}</span><span class="nom">versions</span></div>
</div>

<h2>État de suivi</h2>
<div class="defile">
  <table class="suivi">
    <thead><tr><th>Version</th><th>À relire</th><th>Validés</th><th>Tests validés</th><th>Documents</th></tr></thead>
    <tbody>
      {#each rangement.versions as v (v.nom)}
        {@const s = suivi(v, ids)}
        <tr>
          <th scope="row"><strong>{v.nom}</strong> <span class="discret">{v.titre}</span></th>
          <td class:attention={s.aRelire > 0}>{s.aRelire}</td>
          <td><span class="barre-suivi"><span style="width: {part(s.valides, s.documents)}%"></span></span> {s.valides}</td>
          <td>{s.tests ? `${s.testsValides} / ${s.tests}` : "—"}</td>
          <td>{s.documents}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<h2>Versions</h2>
<ArbreVersions versions={rangement.versions} aRelireIds={ids} ouverte />

{#if rangement.transverses.length}
  <h2>Transverse</h2>
  <ul class="transverses">
    {#each rangement.transverses as d (d.id)}<li><a class="id" href={url.document(d.id)}>{d.id}</a> {d.titre === d.id ? "" : d.titre}</li>{/each}
  </ul>
{/if}
{/if}

<style>
  .defile {
    overflow-x: auto;
  }
  .suivi {
    border-collapse: collapse;
    width: 100%;
    font-size: 14px;
  }
  .suivi th,
  .suivi td {
    text-align: left;
    padding: 6px 10px;
    border-bottom: 1px solid var(--bord);
    white-space: nowrap;
  }
  .suivi thead th {
    color: var(--texte-2);
    font-weight: 600;
    font-size: 12.5px;
  }
  .suivi tbody th .discret {
    font-weight: 400;
    font-size: 13px;
  }
  .attention {
    color: var(--orange);
    font-weight: 650;
  }
  .barre-suivi {
    display: inline-block;
    width: 60px;
    height: 6px;
    border-radius: 3px;
    background: var(--fond-3);
    vertical-align: middle;
    overflow: hidden;
  }
  .barre-suivi span {
    display: block;
    height: 100%;
    background: var(--vert, #2e9e5b);
  }
  .transverses {
    padding-left: 18px;
    font-size: 14px;
  }
</style>
