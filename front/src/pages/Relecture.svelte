<script lang="ts">
  // Documents à relire (EF-142, EF-158) : seulement ceux qui demandent une décision (besoins, exigences,
  // décisions, specs, tests, BDR, cahiers des charges au statut proposé ou brouillon, REC-054), chacun avec
  // son statut, son type et sa version ; « Valider » le passe à validé (Ctrl+Z pour annuler), « Corriger »
  // l'ouvre dans l'éditeur. Filtres par type et par statut.
  import type { Projet } from "../types/Projet";
  import { url } from "../lib/routeur.svelte";
  import { aRelire, changerStatut, signaler } from "../lib/edition.svelte";
  import { terminal } from "../lib/terminal.svelte";

  let { projet }: { projet: Projet | null } = $props();
  /** Statuts qui demandent une relecture, proposés dans le filtre. */
  const STATUTS = ["proposé", "brouillon"];
  /** Documents qui attendent une décision. */
  const docs = $derived(aRelire(projet?.documents ?? []));
  /** Genres présents parmi eux, proposés dans le filtre. */
  const genres = $derived([...new Set(docs.map((d) => d.genre))].sort());
  /** Genre filtré ; vide pour tous. */
  let genre = $state("");
  /** Statut filtré ; vide pour tous. */
  let statut = $state("");
  /** Version d'un document, tirée de son dossier `majeures/<version>/` ; vide hors d'une majeure. */
  const version = (chemin: string) => /majeures\/([^/]+)\//.exec(chemin)?.[1] ?? "";
  /** Documents filtrés, rangés par genre puis par identifiant (tri numérique : EF-10 après EF-9). */
  const liste = $derived(
    docs
      .filter((d) => (!genre || d.genre === genre) && (!statut || d.statut === statut))
      .sort((a, b) => a.genre.localeCompare(b.genre) || a.id.localeCompare(b.id, "fr", { numeric: true })),
  );
  /** Statut posé par « Valider » : `accepté` pour une décision (ADR), `validé` pour le reste. */
  const statutValide = (g: string) => (g === "adr" ? "accepté" : "validé");
  /** Valide un document ; une erreur s'affiche dans le bandeau. */
  async function valider(id: string, g: string) {
    try {
      await changerStatut(id, statutValide(g));
    } catch (e) {
      signaler(String(e instanceof Error ? e.message : e), true);
    }
  }
</script>

<p class="fil"><a href={url.projet()}>Projet</a> · Gestion · à relire</p>
<h1>À relire</h1>
{#if !projet}
  <p class="discret">Chargement…</p>
{:else}
  <p class="discret">Les documents qui attendent ta décision : besoins, exigences, décisions, specs, tests, BDR et cahiers des charges au statut « proposé » ou « brouillon ». Les plans, le backlog et les notes n'y figurent pas. <strong>Valider</strong> passe le document à « validé » (« accepté » pour une décision) ; <strong>Corriger</strong> l'ouvre dans l'éditeur. Ctrl+Z annule.</p>
  <div class="actions">
    <label>Type <select bind:value={genre}><option value="">tous ({docs.length})</option>{#each genres as g}<option value={g}>{g} ({docs.filter((d) => d.genre === g).length})</option>{/each}</select></label>
    <label>Statut <select bind:value={statut}><option value="">tous</option>{#each STATUTS as s}<option value={s}>{s}</option>{/each}</select></label>
    <span class="discret">{liste.length} document(s)</span>
  </div>
  <table class="relecture">
    <thead><tr><th>Statut</th><th>Document</th><th>Type</th><th>Version</th><th></th></tr></thead>
    <tbody>
      {#each liste as d (d.id)}
        <tr>
          <td><span class="pastille statut-{d.statut}">{d.statut}</span></td>
          <td><a href={url.document(d.id)}><span class="id">{d.id}</span> {d.titre === d.id ? "" : d.titre}</a></td>
          <td class="discret">{d.genre}</td>
          <td class="discret">{version(d.chemin)}</td>
          <td class="gestes">
            {#if terminal.proprietaire}
              <button class="valider" onclick={() => valider(d.id, d.genre)} title="Passer à « {statutValide(d.genre)} »">✓ Valider</button>
              <a class="bouton" href={url.document(d.id) + "?editer=1"}>Corriger</a>
            {/if}
          </td>
        </tr>
      {:else}
        <tr><td colspan="5" class="discret">Rien à relire.</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  .gestes {
    white-space: nowrap;
    text-align: right;
  }
  .gestes .valider {
    padding: 3px 10px;
    border: 1px solid var(--vert);
    border-radius: var(--rayon);
    background: var(--vert-fond);
    color: var(--vert);
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .gestes .bouton {
    padding: 3px 10px;
    font-size: 13px;
    margin-left: 4px;
  }
</style>
