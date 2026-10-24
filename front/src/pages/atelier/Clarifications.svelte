<script lang="ts">
  // Clarifications (EF-144, EF-162), présentées comme la relecture (REC-063) : un tableau, une ligne par
  // clarification avec son état, son sujet, sa question et ses pistes gardées ; filtres par état et par
  // sujet ; « Reprendre » et « Supprimer » sur chaque ligne, « Supprimer le sujet » pour toutes celles
  // d'un sujet ; lancer la clarification d'un autre document, trouvé par une recherche dynamique (REC-113).
  // @relu 2026-10-24 6b60b4bb
  import type { Projet } from "../../types/Projet";
  import type { ArtefactResume } from "../../types/ArtefactResume";
  import { atelier, champ, sujets, ecrire } from "../../lib/atelier.svelte";
  import { aller, url, routeur } from "../../lib/routeur.svelte";
  import { signaler, rafraichir } from "../../lib/edition.svelte";
  import ChampFloue from "../../composants/ChampFloue.svelte";

  let { projet }: { projet: Projet | null } = $props();
  /** Titre d'un document par identifiant, pour nommer les sujets. */
  const titreDoc = (id: string) => projet?.documents.find((d) => d.id === id)?.titre ?? "";
  /** Clarifications regroupées par sujet. */
  const groupes = $derived(atelier.etat ? sujets(titreDoc) : []);
  /** Documents qu'on peut faire clarifier : besoins, exigences, storyboards, idées. */
  const candidats = $derived((projet?.documents ?? []).filter((d) => ["besoin", "exigence", "exigence-nf", "storyboard", "idee"].includes(d.genre)));
  /** Document choisi pour lancer une nouvelle clarification. */
  let nouveau = $state("");
  // svelte-ignore state_referenced_locally
  /** Sujet filtré ; `?sujet=` (lien du panneau) le préremplit. */
  let sujet = $state((routeur.route, new URLSearchParams(location.search).get("sujet") ?? ""));
  /** État filtré (« en cours » ou « clarifiée ») ; vide pour tous. */
  let etat = $state("");

  /** État lisible d'une clarification. */
  const etatDe = (c: ArtefactResume) => (champ(c, "etat") === "close" ? "clarifiée" : "en cours");
  /** Lignes du tableau : une par clarification, avec son sujet, après filtres. */
  const lignes = $derived(
    groupes
      .flatMap((g) => g.clarifications.map((c) => ({ g, c })))
      .filter(({ g, c }) => (!sujet || g.sujet === sujet) && (!etat || etatDe(c) === etat)),
  );

  /** Supprime une ou plusieurs clarifications après confirmation, puis relit le projet ; Ctrl+Z les rend. */
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
  <ChampFloue elements={candidats.map((d) => ({ id: d.id, libelle: d.titre, detail: d.genre }))} bind:valeur={nouveau} choisir={(c) => aller(url.clarifier(typeof c === "string" ? c : c.id))} placeholder="Chercher un besoin, une exigence… (ef-140, météo)" libelle="Document à clarifier" />
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
  <thead><tr><th>État</th><th>Clarification</th><th>Sujet</th><th>Pistes</th><th></th></tr></thead>
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
