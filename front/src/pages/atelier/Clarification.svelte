<script lang="ts">
  // Clarification rappelée depuis le développement (SPEC-21 R4) : la question, le sujet, les pistes du LLM
  // qui lit le projet par MCP, puis la conclusion de Tibo.
  // @relu 2026-10-19 aaccaa21
  import { atelier, artefact, champ, ecrire } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Propositions from "../../composants/atelier/Propositions.svelte";
  import Apercu from "../../composants/atelier/Apercu.svelte";

  let { id }: { id: string } = $props();
  const c = $derived(artefact(id));
  const se = $derived(atelier.etat?.fiche_clarification.sous_etapes.find((s) => s.id === "reponse"));
  let conclusion = $state("");
  let erreur = $state("");
  let version = $state(0);

  async function conclure(e: SubmitEvent) {
    e.preventDefault();
    erreur = "";
    try {
      await ecrire("saisir", { etape: "clarification", sous_etape: "conclusion", cible: id, donnees: { conclusion } });
      version++;
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

{#if !c}
  <p class="discret">{atelier.etat ? `Clarification inconnue : ${id}` : "Chargement…"}</p>
{:else}
  <p class="fil"><a href={url.atelier()}>Design thinking</a> · clarification</p>
  <h1>{c.titre}</h1>
  <div class="meta">
    <span class="pastille">{champ(c, "etat")}</span>
    sujet : <a class="pastille" href={url.document(champ(c, "sujet"))}>{champ(c, "sujet")}</a>
  </div>
  <blockquote class="question">{champ(c, "question")}</blockquote>
  <h2>Pistes</h2>
  {#if se}<Propositions etape="clarification" {se} cible={id} mode="express" />{/if}
  <h2>Conclusion</h2>
  {#if champ(c, "conclusion")}
    <p class="texte-libre">{champ(c, "conclusion")}</p>
  {/if}
  <form class="formulaire" onsubmit={conclure}>
    <textarea rows={4} bind:value={conclusion} placeholder="Ce qui est décidé, et ce qu'il faut changer dans la doc ou le code."></textarea>
    <div class="actions"><button class="principal" type="submit" disabled={!conclusion.trim()}>Conclure</button></div>
    {#if erreur}<p class="erreur">{erreur}</p>{/if}
  </form>
  <Apercu {id} {version} />
{/if}
