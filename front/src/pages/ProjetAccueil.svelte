<script lang="ts">
  // Accueil de la section Projet (EF-131, EF-142) : les documents par type, ceux qui attendent une
  // relecture (la tuile mène à la liste complète), et les versions majeures (BDR, cahier des charges, plan).
  // @relu 2026-10-22 8c040df4
  import type { Projet } from "../types/Projet";
  import type { DocumentResume } from "../types/DocumentResume";
  import { url } from "../lib/routeur.svelte";
  import { aRelire as aRelireDe } from "../lib/edition.svelte";

  let { projet }: { projet: Projet | null } = $props();
  /** Base des tuiles et des versions majeures : les documents de projet, sans ceux de l'atelier ni de la réception. */
  const docs = $derived((projet?.documents ?? []).filter((d) => d.zone === "projet"));
  /** Types montrés en tuiles : genre et titre. */
  const TYPES: [string, string][] = [
    ["besoin", "Besoins"],
    ["exigence", "Exigences"],
    ["exigence-nf", "Exigences non fonctionnelles"],
    ["adr", "Décisions (ADR)"],
    ["spec", "Specs"],
  ];
  /** Documents d'un genre. */
  const deType = (g: string) => docs.filter((d) => d.genre === g);
  /** Documents qui attendent une relecture. */
  const aRelire = $derived(aRelireDe(projet?.documents ?? []));
  /** Versions majeures : documents rangés sous `majeures/<version>/`, de la plus récente à la plus ancienne. */
  const majeures = $derived.by(() => {
    const m = new Map<string, DocumentResume[]>();
    for (const d of docs) {
      const v = /majeures\/([^/]+)\//.exec(d.chemin)?.[1];
      if (!v) continue;
      if (!m.has(v)) m.set(v, []);
      m.get(v)!.push(d);
    }
    return [...m.entries()].sort((a, b) => b[0].localeCompare(a[0], "fr", { numeric: true }));
  });
</script>

<h1>Projet</h1>
{#if !projet}<p class="discret">Chargement…</p>{:else}
<p class="discret">Les documents du projet : besoins, exigences, décisions, specs, versions majeures. Ils vivent dans <code>docs/projet/</code>.</p>

<div class="tuiles">
  {#each TYPES as [g, nom]}
    <div class="tuile"><span class="valeur">{deType(g).length}</span><span class="nom">{nom}</span></div>
  {/each}
  <a class="tuile lien-tuile" class:alerte={aRelire.length > 0} href={url.relecture()}><span class="valeur">{aRelire.length}</span><span class="nom">à relire →</span></a>
</div>

{#if majeures.length}
  <h2>Versions majeures</h2>
  <div class="majeures">
    {#each majeures as [v, ds] (v)}
      <section class="majeure">
        <h3>{v}</h3>
        <ul>{#each ds as d (d.id)}<li><a href={url.document(d.id)}>{d.titre}</a> <span class="pastille statut-{d.statut}">{d.statut ?? ""}</span></li>{/each}</ul>
      </section>
    {/each}
  </div>
{/if}

{#if aRelire.length}
  <h2>À relire</h2>
  <ul class="a-relire">
    {#each aRelire.slice(0, 30) as d (d.id)}<li><a class="id" href={url.document(d.id)}>{d.id}</a> {d.titre}</li>{/each}
  </ul>
  {#if aRelire.length > 30}<p><a href={url.relecture()}>Les {aRelire.length} documents à relire, avec leur statut →</a></p>{/if}
{/if}
{/if}
