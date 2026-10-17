<script lang="ts">
  // Tableau de bord de l'atelier : étapes du parcours (état, mode), consommation, clarifications ouvertes.
  // @relu 2026-10-17 2fe7f36a
  import { atelier, PHASES, ETATS } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";

  const etat = $derived(atelier.etat!);
  const faites = $derived(etat.etapes.filter((e) => e.etat === "faite" || e.etat === "sautee").length);
  const prochaine = $derived(etat.etapes.find((e) => e.etat !== "faite" && e.etat !== "sautee"));
  const pct = (x: number) => `${Math.round(x * 100)} %`;
</script>

<h1>Atelier{etat.projet ? ` : ${etat.projet}` : ""}</h1>
<p class="discret">
  Définition du besoin selon le framework Fast Design Thinking. Le LLM ({etat.fournisseur}) lit le projet par MCP et propose ; tu gardes, corriges ou rejettes. Tout est écrit dans <code>docs/atelier/</code> et <code>docs/projet/</code>.
</p>

<div class="tuiles">
  <div class="tuile"><span class="valeur">{faites} / {etat.etapes.length}</span><span class="nom">étapes terminées</span>
    <div class="jauge"><span style="width: {(100 * faites) / etat.etapes.length}%"></span></div></div>
  <div class="tuile"><span class="valeur">{etat.artefacts.length}</span><span class="nom">artefacts</span></div>
  {#if etat.consommation}
    <div class="tuile" class:alerte={etat.consommation.cinq_heures > 0.9}><span class="valeur">{pct(etat.consommation.cinq_heures)}</span><span class="nom">abonnement, 5 h</span></div>
    <div class="tuile" class:alerte={etat.consommation.sept_jours > 0.9}><span class="valeur">{pct(etat.consommation.sept_jours)}</span><span class="nom">abonnement, 7 jours</span></div>
  {/if}
</div>

{#if prochaine}
  <p><a class="bouton principal" href={url.etape(prochaine.fiche.id)}>{prochaine.etat === "a_faire" ? "Commencer" : "Reprendre"} : {prochaine.fiche.titre} →</a></p>
{/if}

<ol class="stepper">
  {#each etat.etapes as e, i (e.fiche.id)}
    <li class="pas {e.etat}">
      <span class="numero">{e.etat === "faite" ? "✓" : i + 1}</span>
      <a href={url.etape(e.fiche.id)}>
        <strong>{e.fiche.titre}</strong>
        <span class="discret">{PHASES[e.fiche.phase] ?? e.fiche.phase}{e.fiche.optionnelle ? " · optionnelle" : ""} · {e.mode}</span>
      </a>
      <span class="pastille etat-{e.etat}">{ETATS[e.etat] ?? e.etat}</span>
      {#if e.artefacts.length}<span class="compteur">{e.artefacts.length}</span>{/if}
    </li>
  {/each}
</ol>

<p class="discret">
  Le wiki du projet est servi ici aussi (barre de gauche, recherche, graphe). Pour le consulter à part : <a href={etat.url_wiki}>{etat.url_wiki}</a> (<code>wikik serve</code>).
</p>

<h2>Clarifications</h2>
{#if etat.clarifications.length}
  <ul>
    {#each etat.clarifications as c (c.id)}
      <li><a href={url.clarification(c.id)}>{c.id}</a> {c.titre} <span class="pastille">{c.frontmatter.etat ?? ""}</span></li>
    {/each}
  </ul>
{:else}
  <p class="discret">Aucune. Pendant le développement, l'agent en ouvre une avec l'outil MCP <code>atelier_rappel</code> quand un point du besoin est flou.</p>
{/if}
