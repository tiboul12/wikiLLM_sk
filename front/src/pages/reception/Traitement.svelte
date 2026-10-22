<script lang="ts">
  // Traitement des remarques (SPEC-23 R5) : les confier à l'agent dans le terminal (EF-148), ou filtrer
  // par statut, gravité et page, puis passer chacune à traitée, reportée ou rejetée avec une explication.
  // « À valider » : les remarques traitées que tu n'as pas encore validées (EF-165) ; la légende dit la
  // couleur de chaque version publiée.
  import { reception, actuelles, GRAVITES, STATUTS, chargerGit } from "../../lib/reception.svelte";
  import LegendeVersions from "../../composants/reception/LegendeVersions.svelte";
  import { url } from "../../lib/routeur.svelte";
  import CarteRemarque from "../../composants/reception/CarteRemarque.svelte";
  import EnvoiAgent from "../../composants/reception/EnvoiAgent.svelte";

  let statut = $state("ouverte");
  chargerGit();
  // Filtres de validation, en plus des statuts.
  const FILTRES: Record<string, (r: { statut: string; validation: string | null }) => boolean> = {
    a_valider: (r) => r.statut === "traitee" && r.validation !== "validee",
    validee: (r) => r.validation === "validee",
  };
  let gravite = $state("");
  let page = $state("");
  const rang = (g: string) => ["bloquant", "genant", "detail"].indexOf(g);
  const liste = $derived(
    actuelles()
      .filter((r) => (!statut || (FILTRES[statut] ? FILTRES[statut](r) : r.statut === statut)) && (!gravite || r.gravite === gravite) && (!page || r.page === page))
      .sort((a, b) => rang(a.gravite) - rang(b.gravite) || a.id.localeCompare(b.id)),
  );
  const pagesAvecRemarques = $derived([...new Map(actuelles().map((r) => [r.page, r.nom_page])).entries()]);
  const compte = (s: string) => actuelles().filter((r) => (FILTRES[s] ? FILTRES[s](r) : r.statut === s)).length;
</script>

<p class="fil"><a href={url.reception()}>Cahier de réception</a> · traitement</p>
<h1>Traitement des remarques</h1>
<div class="actions"><EnvoiAgent /></div>
<div class="filtres">
  <span class="bascule" role="group" aria-label="Statut">
    {#each Object.entries(STATUTS) as [v, nom]}<button class:actif={statut === v} onclick={() => (statut = v)}>{nom} <small>{compte(v)}</small></button>{/each}
    <button class:actif={statut === "a_valider"} onclick={() => (statut = "a_valider")}>À valider <small>{compte("a_valider")}</small></button>
    <button class:actif={statut === "validee"} onclick={() => (statut = "validee")}>Validées <small>{compte("validee")}</small></button>
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
<LegendeVersions />
{#each liste as r (r.id)}<CarteRemarque {r} traitement={r.statut !== "traitee" || statut !== "a_valider"} avecPage />{/each}
