<script lang="ts">
  // Clarifications (EF-144, EF-162), présentées comme la relecture (REC-063) : un tableau, une ligne par
  // clarification avec son état, son sujet, sa question et ses pistes gardées ; filtres par état et par
  // sujet ; « Reprendre » et « Supprimer » sur chaque ligne, « Supprimer le sujet » pour toutes celles
  // d'un sujet ; lancer la clarification d'un autre document.
  import type { Projet } from "../../types/Projet";
  import type { ArtefactResume } from "../../types/ArtefactResume";
  import { atelier, champ, sujets, ecrire } from "../../lib/atelier.svelte";
  import { aller, url, routeur } from "../../lib/routeur.svelte";
  import { signaler, rafraichir } from "../../lib/edition.svelte";

  let { projet }: { projet: Projet | null } = $props();
  const titreDoc = (id: string) => projet?.documents.find((d) => d.id === id)?.titre ?? "";
  const groupes = $derived(atelier.etat ? sujets(titreDoc) : []);
  const candidats = $derived((projet?.documents ?? []).filter((d) => ["besoin", "exigence", "exigence-nf", "storyboard", "idee"].includes(d.genre)));
  let nouveau = $state("");
  // svelte-ignore state_referenced_locally
  let sujet = $state((routeur.route, new URLSearchParams(location.search).get("sujet") ?? ""));
  let etat = $state("");

  const etatDe = (c: ArtefactResume) => (champ(c, "etat") === "close" ? "clarifiée" : "en cours");
  const lignes = $derived(
    groupes
      .flatMap((g) => g.clarifications.map((c) => ({ g, c })))
      .filter(({ g, c }) => (!sujet || g.sujet === sujet) && (!etat || etatDe(c) === etat)),
  );

  async function supprimer(ids: string[], quoi: string) {
    if (!confirm(`Supprimer ${quoi} et ses pistes ? (Ctrl+Z pour annuler)`)) return;
    try {
      for (const id of ids) await ecrire("clarification", { id, action: "supprimer" });
      signaler(`${quoi} supprimé${ids.length > 1 ? "es" : "e"} (Ctrl+Z pour annuler)`);
      rafraichir();
    } catch (e) {
      signaler(String(e instanceof Error ? e.message : e), true);
    }
  }
</script>

<p class="fil">Design thinking · clarifications</p>
<h1>Clarifications</h1>
<p class="discret">
  Les points à arbitrer : quand un besoin, une exigence ou un storyboard est flou, le LLM juge la question puis propose des pistes ; tu gardes, corriges ou rejettes. L'agent de code en ouvre aussi pendant le développement (outil <code>atelier_rappel</code>).
</p>

<form class="actions" onsubmit={(e) => { e.preventDefault(); if (nouveau.trim()) aller(url.clarifier(nouveau.trim())); }}>
  <input list="sujets-possibles" bind:value={nouveau} placeholder="B-001, EF-140…" aria-label="Document à clarifier" />
  <datalist id="sujets-possibles">{#each candidats as d (d.id)}<option value={d.id}>{d.titre}</option>{/each}</datalist>
  <button class="principal" type="submit" disabled={!nouveau.trim()}>Clarifier</button>
</form>

<div class="actions">
  <label>Sujet <select bind:value={sujet}><option value="">tous ({groupes.length})</option>{#each groupes as g (g.sujet)}<option value={g.sujet}>{g.sujet} {g.titre}</option>{/each}</select></label>
  <label>État <select bind:value={etat}><option value="">tous</option><option value="en cours">en cours</option><option value="clarifiée">clarifiée</option></select></label>
  <span class="discret">{lignes.length} clarification(s)</span>
  {#if sujet}
    {@const g = groupes.find((x) => x.sujet === sujet)}
    {#if g}<button class="discret-bouton danger-texte" onclick={() => supprimer(g.clarifications.map((c) => c.id), `toutes les clarifications de ${sujet}`)}>Supprimer le sujet</button>{/if}
  {/if}
</div>

<table class="relecture">
  <thead><tr><th>État</th><th>Clarification</th><th>Sujet</th><th>Pistes gardées</th><th></th></tr></thead>
  <tbody>
    {#each lignes as { g, c } (c.id)}
      <tr>
        <td><span class="pastille" class:statut-retenu={etatDe(c) === "clarifiée"}>{etatDe(c)}</span></td>
        <td><a href={url.clarification(c.id)}><span class="id">{c.id}</span> {champ(c, "question")}</a></td>
        <td><a href={url.document(g.sujet)} title={g.titre}>{g.sujet}</a></td>
        <td class="discret">{((c.frontmatter.pistes as unknown[]) ?? []).length}</td>
        <td class="gestes">
          <a class="bouton" href={url.clarification(c.id)}>{etatDe(c) === "clarifiée" ? "Voir" : "Reprendre"}</a>
          <button class="discret-bouton danger-texte" onclick={() => supprimer([c.id], c.id)} title="Supprimer cette clarification">Supprimer</button>
        </td>
      </tr>
    {:else}
      <tr><td colspan="5" class="discret">Aucune clarification.</td></tr>
    {/each}
  </tbody>
</table>

<style>
  td:first-child,
  td:nth-child(3),
  td:nth-child(4) {
    white-space: nowrap;
  }
  .gestes {
    white-space: nowrap;
    text-align: right;
  }
  .gestes .bouton {
    padding: 3px 10px;
    font-size: 13px;
  }
  .danger-texte {
    color: var(--rouge);
  }
</style>
