<script lang="ts">
  // « Traiter avec l'agent » (REC-041, EF-148, EF-156) : envoie le skill de traitement (Paramètres,
  // Prompts système) dans le terminal intégré ; collé dans l'agent s'il tourne, sinon l'agent est lancé
  // avec lui.
  import { terminal, traiterAvecAgent } from "../../lib/terminal.svelte";
  import { url } from "../../lib/routeur.svelte";

  /** Ce qui a été fait : agent lancé avec le skill, ou skill collé dans l'agent qui tourne. */
  let message = $state("");
  /** Erreur d'envoi (terminal indisponible, visiteur non propriétaire). */
  let erreur = $state("");
  /** Envoie le skill de traitement de la réception dans le terminal intégré. */
  async function traiter() {
    message = erreur = "";
    try {
      const r = await traiterAvecAgent();
      message = r === "agent" ? `Agent lancé avec le skill de traitement (${terminal.parametres?.agent}).` : "Skill de traitement envoyé à l'agent, dans le terminal.";
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

{#if terminal.proprietaire && terminal.parametres}
  <button class="principal" onclick={traiter} title="L'agent reçoit le skill wikik-traiter-reception (Paramètres, Prompts système) : il commence par un plan">Traiter avec l'agent ↗</button>
  {#if message}<span class="succes">{message}</span>{/if}
  {#if erreur}<span class="erreur">{erreur}</span>{/if}
{:else}
  <a class="bouton principal" href={url.parametres()} title="Le terminal est réservé au propriétaire du projet">Traiter avec l'agent : accès propriétaire</a>
{/if}
