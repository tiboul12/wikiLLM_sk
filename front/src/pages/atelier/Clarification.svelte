<script lang="ts">
  // Clarification d'un sujet (SPEC-21 R4, EF-144) : le sujet, la question, les pistes que le LLM propose en
  // lisant le projet par MCP (demandées d'office à l'ouverture) ; garder, corriger ou rejeter fait la
  // réponse ; « Clarifié » la clôt avec les pistes gardées. Se reprend, se rouvre, se supprime. Une question
  // jugée hors sujet par le LLM est signalée en tête (EF-162).
  // @ecrans /clarifications/:id Clarification d'un sujet
  import { atelier, artefact, champ, ecrire } from "../../lib/atelier.svelte";
  import { aller, url } from "../../lib/routeur.svelte";
  import Propositions from "../../composants/atelier/Propositions.svelte";

  let { id }: { id: string } = $props();
  const c = $derived(artefact(id) ?? atelier.etat?.clarifications.find((x) => x.id === id));
  const se = $derived(atelier.etat?.fiche_clarification.sous_etapes.find((s) => s.id === "reponse"));
  const lot = $derived(atelier.etat?.lots_clarification.find((l) => l.cible === id));
  const gardees = $derived(((c?.frontmatter.pistes as { piste: string }[] | undefined) ?? []).map((p) => p.piste));
  const close = $derived(champ(c, "etat") === "close");
  // Le LLM juge d'abord la question (REC-064) : hors sujet, il le dit en tête de ses pistes.
  const horsSujet = $derived(lot?.elements.map((p) => String((p.donnees as Record<string, unknown>).hors_sujet ?? "")).find((t) => t.trim()) ?? "");
  const sujet = $derived(champ(c, "sujet"));
  let erreur = $state("");

  // Les pistes sont demandées dès l'ouverture d'une clarification qui n'en a pas encore (REC-024.2).
  let demandee = false;
  $effect(() => {
    if (!c || !se || close || demandee || lot?.elements.length || atelier.occupe) return;
    demandee = true;
    ecrire("proposer", { etape: "clarification", sous_etape: "reponse", cible: id }, true).catch((x) => (erreur = String(x instanceof Error ? x.message : x)));
  });

  async function agir(action: "clore" | "rouvrir" | "supprimer") {
    if (action === "supprimer" && !confirm(`Supprimer ${id} et ses pistes ?`)) return;
    erreur = "";
    try {
      await ecrire("clarification", { id, action });
      if (action === "supprimer") aller(url.clarifications(), true);
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

{#if !c}
  <p class="discret">{atelier.etat ? `Clarification inconnue : ${id}` : "Chargement…"}</p>
{:else}
  <p class="fil"><a href={url.clarifications()}>Clarifications</a> · <a href={url.clarifications() + "?sujet=" + encodeURIComponent(sujet)}>{sujet}</a> · {id}</p>
  <h1>Clarifier <a href={url.document(sujet)}>{sujet}</a></h1>
  <div class="meta">
    <span class="pastille" class:statut-retenu={close}>{close ? "clarifiée" : "en cours"}</span>
  </div>
  <blockquote class="question">{champ(c, "question")}</blockquote>

  <h2>Pistes</h2>
  <p class="discret">Le LLM lit le sujet, ses sources et le projet. Garde, corrige ou rejette : les pistes gardées font la réponse.</p>
  {#if horsSujet}
    <div class="hors-sujet" role="note"><strong>Question hors sujet</strong> selon le LLM : {horsSujet}</div>
  {/if}
  {#if se}<Propositions etape="clarification" {se} cible={id} mode="express" />{/if}

  <h2>Réponse</h2>
  {#if gardees.length}
    <ul class="reponse-clarification">{#each gardees as g}<li>{g}</li>{/each}</ul>
  {:else}
    <p class="discret">Aucune piste gardée pour l'instant.</p>
  {/if}
  <div class="actions fin-etape">
    {#if close}
      <button onclick={() => agir("rouvrir")}>Rouvrir</button>
    {:else}
      <button class="principal" disabled={!gardees.length} onclick={() => agir("clore")} title={gardees.length ? "" : "Garde au moins une piste"}>Clarifié : terminer</button>
    {/if}
    <button class="danger" onclick={() => agir("supprimer")}>Supprimer</button>
  </div>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
{/if}

<style>
  .hors-sujet {
    padding: 10px 14px;
    margin: 8px 0 12px;
    border: 1px solid var(--orange);
    border-radius: var(--rayon);
    background: var(--orange-fond);
    color: var(--texte);
  }
</style>
