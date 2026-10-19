<script lang="ts">
  // Section Paramètres (EF-147) : l'agent de développement et la consigne de traitement des remarques,
  // la méthode de lecture du wiki donnée aux LLM, le LLM de l'atelier et de la réception, l'adresse de
  // l'application reçue. Écrit dans wikik.toml ; réservé au propriétaire (ADR-0013).
  import type { Parametres } from "../types/Parametres";
  import { terminal, chargerParametres, enregistrerParametres } from "../lib/terminal.svelte";
  import { reception, ecrireReception } from "../lib/reception.svelte";

  let p = $state<Parametres | null>(null);
  let arguments_ = $state("");
  let adresse = $state("");
  let message = $state("");
  let erreur = $state("");

  chargerParametres().then(() => {
    if (!terminal.parametres) return;
    p = structuredClone($state.snapshot(terminal.parametres));
    arguments_ = p.arguments.join("\n");
    adresse = reception.etat?.url_application ?? "";
  });

  async function enregistrer(e: SubmitEvent) {
    e.preventDefault();
    if (!p) return;
    message = erreur = "";
    try {
      await enregistrerParametres({ ...p, arguments: arguments_.split("\n").map((a) => a.trim()).filter((a, i, t) => a || i < t.length - 1) });
      if (reception.actif && adresse.trim() !== (reception.etat?.url_application ?? "")) await ecrireReception("application", { url: adresse.trim() });
      message = "Enregistré dans wikik.toml.";
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

<h1>Paramètres</h1>
<p class="discret">Réglages du projet, écrits dans <code>wikik.toml</code> (une valeur laissée par défaut n'y est pas écrite). Le LLM change sans relancer wikik.</p>
{#if !p}
  <p class="discret">Chargement…</p>
{:else}
  {#if !terminal.proprietaire}
    <p class="alerte-acces">Lecture seule : modifier ces réglages (et ouvrir le terminal) est réservé au propriétaire. Ouvre wikik depuis sa machine, ou avec l'adresse à clé affichée au lancement de <code>wikik</code>.</p>
  {/if}
  <form class="formulaire parametres" onsubmit={enregistrer}>
    <fieldset disabled={!terminal.proprietaire}>
      <h2>Agent de développement</h2>
      <label><span>Commande de l'agent</span><input bind:value={p.agent} placeholder="claude" /><small>Lancée dans le terminal intégré par « Lancer l'agent » et par « Traiter avec l'agent ».</small></label>
      <label><span>Consigne de traitement des remarques</span><textarea rows={5} bind:value={p.consigne_traitement}></textarea><small>Envoyée à l'agent par « Traiter avec l'agent » (cahier de réception). Par défaut, elle l'oblige à commencer par un plan.</small></label>

      <h2>Lecture du wiki par les LLM</h2>
      <label><span>Méthode de lecture</span><textarea rows={4} bind:value={p.consigne_lecture}></textarea><small>Instructions du serveur MCP de wikik (Claude Code les reçoit) et préambule des demandes du Design thinking et de la réception.</small></label>

      <h2>LLM du Design thinking et de la réception</h2>
      {#if terminal.factice}<p class="discret">wikik tourne avec <code>--factice</code> : ces réglages serviront au prochain lancement sans.</p>{/if}
      <label><span>Fournisseur</span>
        <select bind:value={p.fournisseur}><option value="cli">cli : un LLM en ligne de commande</option><option value="factice">factice : exemples des fiches, sans LLM</option></select>
      </label>
      <label><span>Commande</span><input bind:value={p.commande} /></label>
      <label><span>Arguments (un par ligne)</span><textarea rows={8} class="mono" bind:value={arguments_}></textarea><small>Jetons remplacés à chaque appel : <code>{"{consigne}"}</code>, <code>{"{schema}"}</code>, <code>{"{mcp}"}</code>.</small></label>
      <label><span>Délai maximum (s)</span><input type="number" min="10" bind:value={p.delai} /></label>

      <h2>Application reçue</h2>
      <label><span>Adresse de l'application</span><input bind:value={adresse} placeholder={reception.etat?.url_detectee === "/" ? "ce serveur (déduit)" : (reception.etat?.url_detectee ?? "http://10.0.0.21:4200")} /><small>Vide : déduite du projet ({reception.etat?.url_detectee === "/" ? "ce serveur" : (reception.etat?.url_detectee ?? "rien de trouvé")}).</small></label>

      <div class="actions">
        <button class="principal" type="submit">Enregistrer</button>
        {#if message}<span class="succes">{message}</span>{/if}
      </div>
      {#if erreur}<p class="erreur">{erreur}</p>{/if}
    </fieldset>
  </form>
{/if}
