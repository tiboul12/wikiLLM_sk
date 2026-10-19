<script lang="ts">
  // Ouvrir une clarification sur un document (SPEC-21) : depuis le wiki, ou quand un point du besoin est flou.
  // @relu 2026-10-19 724eee5c
  import { api } from "../../lib/api";
  import { ecrire } from "../../lib/atelier.svelte";
  import { aller, url } from "../../lib/routeur.svelte";
  import type { PageDocument } from "../../types/PageDocument";

  let { sujet }: { sujet: string } = $props();
  let doc = $state<PageDocument | null>(null);
  let question = $state("");
  let erreur = $state("");
  // svelte-ignore state_referenced_locally
  api.document(sujet).then((d) => (doc = d)).catch(() => (doc = null));

  async function ouvrir(e: SubmitEvent) {
    e.preventDefault();
    erreur = "";
    try {
      const [id] = await ecrire<[string, string]>("rappel", { sujet, question });
      aller(url.clarification(id));
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

<p class="fil"><a href={url.atelier()}>Design thinking</a> · nouvelle clarification</p>
<h1>Clarifier <a href={url.document(sujet)}>{sujet}</a></h1>
{#if doc}<p class="discret">{doc.genre} · {doc.titre}</p>{/if}
<form class="formulaire" onsubmit={ouvrir}>
  <label>
    <span>Question</span>
    <textarea rows={4} bind:value={question} placeholder="Ce qui est flou : un seuil, un cas limite, une contradiction…"></textarea>
  </label>
  <div class="actions"><button class="principal" type="submit" disabled={!question.trim()}>Ouvrir la clarification</button></div>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
</form>
