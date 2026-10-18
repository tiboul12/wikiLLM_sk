<script lang="ts">
  // Traitement des remarques (SPEC-23 R5) : filtrer par statut, gravité et page, puis passer chacune à
  // traitée, reportée ou rejetée avec une explication.
  import { reception, GRAVITES, STATUTS } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";
  import CarteRemarque from "../../composants/reception/CarteRemarque.svelte";

  let statut = $state("ouverte");
  let gravite = $state("");
  let page = $state("");
  const rang = (g: string) => ["bloquant", "genant", "detail"].indexOf(g);
  const liste = $derived(
    (reception.etat?.remarques ?? [])
      .filter((r) => (!statut || r.statut === statut) && (!gravite || r.gravite === gravite) && (!page || r.page === page))
      .sort((a, b) => rang(a.gravite) - rang(b.gravite) || a.id.localeCompare(b.id)),
  );
  const pagesAvecRemarques = $derived([...new Map((reception.etat?.remarques ?? []).map((r) => [r.page, r.nom_page])).entries()]);
  const compte = (s: string) => (reception.etat?.remarques ?? []).filter((r) => r.statut === s).length;
</script>

<p class="fil"><a href={url.reception()}>Réception</a> · traitement</p>
<h1>Traitement des remarques</h1>
<div class="filtres">
  <span class="bascule" role="group" aria-label="Statut">
    {#each Object.entries(STATUTS) as [v, nom]}<button class:actif={statut === v} onclick={() => (statut = v)}>{nom} <small>{compte(v)}</small></button>{/each}
    <button class:actif={!statut} onclick={() => (statut = "")}>Toutes</button>
  </span>
  <select bind:value={gravite} aria-label="Gravité">
    <option value="">toutes gravités</option>
    {#each Object.entries(GRAVITES) as [v, nom]}<option value={v}>{nom}</option>{/each}
  </select>
  <select bind:value={page} aria-label="Page">
    <option value="">toutes les pages</option>
    {#each pagesAvecRemarques as [cle, nom]}<option value={cle}>{nom}</option>{/each}
  </select>
</div>
<p class="discret">
  {liste.length} remarque(s). L'agent de code les lit aussi avec l'outil MCP <code>reception</code> et enregistre ce qu'il a fait avec <code>reception_traiter</code>.
</p>
{#each liste as r (r.id)}<CarteRemarque {r} traitement avecPage />{/each}
