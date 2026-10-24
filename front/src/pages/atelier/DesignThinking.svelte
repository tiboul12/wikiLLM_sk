<script lang="ts">
  // Accueil du Design thinking (l'atelier, EF-140, EF-145) : le parcours affiché (étapes, état, mode),
  // les autres parcours du projet, la consommation ; les clarifications ont leur tableau de bord.
  // @relu 2026-10-19 4bfd1579
  import { atelier, PHASES, ETATS } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";

  const etat = $derived(atelier.etat!);
  const faites = $derived(etat.etapes.filter((e) => e.etat === "faite" || e.etat === "sautee").length);
  const prochaine = $derived(etat.etapes.find((e) => e.etat !== "faite" && e.etat !== "sautee"));
  const courant = $derived(etat.tous_parcours.find((p) => p.id === etat.parcours));
  const pct = (x: number) => `${Math.round(x * 100)} %`;
</script>

<p class="fil">Design thinking · {courant?.id ?? etat.parcours} · version {courant?.version ?? ""}</p>
<h1>{courant?.titre ?? "Parcours"}{etat.projet ? ` : ${etat.projet}` : ""}</h1>
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
  <p><a class="bouton principal" href={url.etape(prochaine.fiche.id, etat.parcours)}>{prochaine.etat === "a_faire" ? "Commencer" : "Reprendre"} : {prochaine.fiche.titre} →</a></p>
{/if}

<ol class="stepper">
  {#each etat.etapes as e, i (e.fiche.id)}
    <li class="pas {e.etat}">
      <span class="numero">{e.etat === "faite" ? "✓" : i + 1}</span>
      <a href={url.etape(e.fiche.id, etat.parcours)}>
        <strong>{e.fiche.titre}</strong>
        <span class="discret">{PHASES[e.fiche.phase] ?? e.fiche.phase}{e.fiche.optionnelle ? " · optionnelle" : ""} · {e.mode}</span>
      </a>
      <span class="pastille etat-{e.etat}">{ETATS[e.etat] ?? e.etat}</span>
      {#if e.artefacts.length}<span class="compteur">{e.artefacts.length}</span>{/if}
    </li>
  {/each}
</ol>

{#if etat.tous_parcours.length > 1}
  <h2>Autres parcours</h2>
  <ul>
    {#each etat.tous_parcours.filter((p) => p.id !== etat.parcours) as p (p.id)}
      <li><a href={url.atelier(p.id)}>{p.titre}</a> <span class="discret">{p.id} · version {p.version} · {p.faites} / {p.total} étapes</span></li>
    {/each}
  </ul>
{/if}
<p class="discret">Un nouveau parcours (une autre version, un autre sujet) : « ＋ » dans le volet. Les clarifications sont dans la section Projet : <a href={url.clarifications()}>tableau de bord</a>.</p>
