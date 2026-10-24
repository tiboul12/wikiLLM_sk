<script lang="ts">
  // « Traiter avec l'agent » (REC-041, EF-148) : envoie la consigne de traitement (Paramètres) dans le
  // terminal intégré ; collée dans l'agent s'il tourne, sinon l'agent est lancé avec elle.
  import { terminal, envoyer } from "../../lib/terminal.svelte";
  import { url } from "../../lib/routeur.svelte";

  let message = $state("");
  let erreur = $state("");
  async function traiter() {
    message = erreur = "";
    try {
      const r = await envoyer(terminal.parametres?.consigne_traitement ?? "", "consigne");
      message = r === "agent" ? `Agent lancé avec la consigne de traitement (${terminal.parametres?.agent}).` : "Consigne envoyée à l'agent, dans le terminal.";
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

{#if terminal.proprietaire && terminal.parametres}
  <button class="principal" onclick={traiter} title="Consigne réglable dans les Paramètres : {terminal.parametres.consigne_traitement}">Traiter avec l'agent ↗</button>
  {#if message}<span class="succes">{message}</span>{/if}
  {#if erreur}<span class="erreur">{erreur}</span>{/if}
{:else}
  <a class="bouton principal" href={url.parametres()} title="Le terminal est réservé au propriétaire du projet">Traiter avec l'agent : accès propriétaire</a>
{/if}
