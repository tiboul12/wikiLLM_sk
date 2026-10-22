<script lang="ts">
  // Accueil du Design thinking (EF-140, EF-145, EF-161) : deux parties, les ateliers et les clarifications.
  // L'atelier affiché est une session (numéro, titre, version, statut, modifiables) avec ses étapes ; les
  // autres ateliers du projet ; la consommation ; les clarifications ont leur tableau.
  import { atelier, PHASES, ETATS, STATUTS_ATELIER, ecrire } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";
  import { signaler } from "../../lib/edition.svelte";
  import { terminal } from "../../lib/terminal.svelte";

  /** État du parcours affiché ; la page n'est montrée qu'une fois l'atelier chargé. */
  const etat = $derived(atelier.etat!);
  /** Étapes terminées ou sautées, pour l'avancement. */
  const faites = $derived(etat.etapes.filter((e) => e.etat === "faite" || e.etat === "sautee").length);
  /** Première étape ni faite ni sautée : le bouton « Continuer » y mène. */
  const prochaine = $derived(etat.etapes.find((e) => e.etat !== "faite" && e.etat !== "sautee"));
  /** Atelier affiché parmi ceux du projet (titre, version, statut). */
  const courant = $derived(etat.tous_parcours.find((p) => p.id === etat.parcours));
  /** Fraction de 0 à 1 en pourcentage entier, pour la consommation de quota. */
  const pct = (x: number) => `${Math.round(x * 100)} %`;

  /** Formulaire de modification de l'atelier ouvert (REC-073). */
  let edition = $state(false);
  /** Titre de l'atelier en cours de modification. */
  let titre = $state("");
  /** Version documentée par l'atelier. */
  let version = $state("");
  /** Statut de session choisi ; vide, il est déduit des étapes. */
  let statut = $state("");
  /** Ouvre le formulaire prérempli avec l'atelier affiché ; le statut repart vide (déduit). */
  function ouvrirEdition() {
    titre = courant?.titre ?? "";
    version = courant?.version ?? "";
    statut = "";
    edition = true;
  }
  /** Enregistre titre, version et statut de l'atelier ; Ctrl+Z annule. */
  async function enregistrer(e: SubmitEvent) {
    e.preventDefault();
    try {
      await ecrire("parcours/modifier", { titre, version, statut });
      edition = false;
      signaler("Atelier modifié (Ctrl+Z pour annuler)");
    } catch (x) {
      signaler(String(x instanceof Error ? x.message : x), true);
    }
  }
</script>

<p class="fil">Design thinking · ateliers · n° {courant?.numero ?? 1}</p>
<h1>Atelier n° {courant?.numero ?? 1} : {courant?.titre ?? "Parcours"}{etat.projet ? ` · « ${etat.projet} »` : ""}</h1>
<div class="meta-atelier">
  <span class="cle">version</span> <span class="pastille">{courant?.version ?? ""}</span>
  <span class="cle">statut</span> <span class="pastille"><span class="statut-atelier statut-{courant?.statut}"></span> {STATUTS_ATELIER[courant?.statut ?? ""] ?? courant?.statut}</span>
  <span class="cle">identifiant</span> <code>{courant?.id}</code>
  {#if terminal.proprietaire && !edition}<button class="discret-bouton" onclick={ouvrirEdition}>✎ Modifier l'atelier</button>{/if}
</div>
{#if edition}
  <form class="formulaire edition-atelier" onsubmit={enregistrer}>
    <label><span>Titre</span><input bind:value={titre} required /></label>
    <label><span>Version documentée</span><input bind:value={version} placeholder="v1" /></label>
    <label><span>Statut</span>
      <select bind:value={statut}>
        <option value="">déduit des étapes</option>
        {#each Object.entries(STATUTS_ATELIER) as [k, v]}<option value={k}>{v}</option>{/each}
      </select>
    </label>
    <div class="actions"><button class="principal" type="submit">Enregistrer</button><button type="button" onclick={() => (edition = false)}>Annuler</button></div>
  </form>
{/if}
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

<h2>Tous les ateliers</h2>
<table class="relecture">
  <thead><tr><th>N°</th><th>Atelier</th><th>Version</th><th>Statut</th><th>Étapes</th></tr></thead>
  <tbody>
    {#each etat.tous_parcours as p (p.id)}
      <tr class:courant={p.id === etat.parcours}>
        <td>{p.numero}</td>
        <td><a href={url.atelier(p.id)}>{p.titre}</a></td>
        <td class="discret">{p.version}</td>
        <td><span class="statut-atelier statut-{p.statut}"></span> {STATUTS_ATELIER[p.statut] ?? p.statut}</td>
        <td class="discret">{p.faites} / {p.total}</td>
      </tr>
    {/each}
  </tbody>
</table>
<p class="discret">Un nouvel atelier (une autre version, un autre sujet) : « ＋ » dans le volet. Les points à arbitrer sont dans la seconde partie du Design thinking : <a href={url.clarifications()}>les clarifications</a> ({etat.clarifications.length}).</p>

<style>
  .meta-atelier {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    margin: -4px 0 14px;
    font-size: 13.5px;
  }
  .meta-atelier .cle {
    color: var(--texte-2);
  }
  .edition-atelier {
    max-width: 520px;
    margin-bottom: 16px;
  }
  tr.courant td {
    font-weight: 600;
  }
</style>
