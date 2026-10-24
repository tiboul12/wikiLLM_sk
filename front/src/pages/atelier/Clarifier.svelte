<script lang="ts">
  // Lancer la clarification d'un document (EF-144, REC-024.2) : rien à écrire ; la clarification ouverte
  // sur ce sujet est reprise, sinon une nouvelle est créée, et l'on arrive directement sur ses pistes.
  import { ecrire } from "../../lib/atelier.svelte";
  import { aller, url } from "../../lib/routeur.svelte";

  let { sujet }: { sujet: string } = $props();
  let erreur = $state("");
  // svelte-ignore state_referenced_locally
  ecrire<[string, string]>("rappel", { sujet })
    .then(([id]) => aller(url.clarification(id), true))
    .catch((x) => (erreur = String(x instanceof Error ? x.message : x)));
</script>

<p class="fil"><a href={url.clarifications()}>Clarifications</a> · {sujet}</p>
{#if erreur}
  <p class="erreur">{erreur}</p>
{:else}
  <p class="discret">Ouverture de la clarification de {sujet}…</p>
{/if}
