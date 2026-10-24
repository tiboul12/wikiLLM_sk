<script lang="ts">
  // Écran d'une étape du parcours affiché : « en bref » (à quoi elle sert, ce qu'on fait, ce qu'elle produit), règles de la
  // fiche, mode, puis chaque sous-étape avec son écran (dédié pour le mode Express, générique sinon) ; la
  // carte mentale se modifie dans l'éditeur Mermaid (EF-167).
  // Une page de réception par étape (EF-164) :
  // @ecrans /atelier/pitch Pitch
  // @ecrans /atelier/free-writing Free writing
  // @ecrans /atelier/carte-mentale Carte mentale
  // @ecrans /atelier/analogies Pensée analogique
  // @ecrans /atelier/5w1h 5W1H
  // @ecrans /atelier/cubing Cubing
  // @ecrans /atelier/scamper SCAMPER
  // @ecrans /atelier/storyboard Storyboard
  // @ecrans /atelier/bilan Bilan des besoins
  // @ecrans /atelier/exigences Exigences
  // @ecrans /atelier/roadmap Roadmap
  import type { SousEtape } from "../../types/SousEtape";
  import { atelier, deType, ecrire, PHASES, ETATS } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Propositions from "../../composants/atelier/Propositions.svelte";
  import Selection from "../../composants/atelier/Selection.svelte";
  import Synthese from "../../composants/atelier/Synthese.svelte";
  import Pitch from "../../composants/atelier/Pitch.svelte";
  import FreeWriting from "../../composants/atelier/FreeWriting.svelte";
  import Tri from "../../composants/atelier/Tri.svelte";
  import EditeurMermaid from "../../composants/atelier/EditeurMermaid.svelte";
  import { terminal } from "../../lib/terminal.svelte";
  import Planche from "../../composants/atelier/Planche.svelte";
  import EnBref from "../../composants/atelier/EnBref.svelte";

  let { etape }: { etape: string } = $props();
  /** Rang de l'étape dans le parcours affiché ; -1 si elle n'y est pas. */
  const index = $derived(atelier.etat?.etapes.findIndex((e) => e.fiche.id === etape) ?? -1);
  /** Étape affichée : fiche, état, mode, lots et sélections. */
  const e = $derived(atelier.etat?.etapes[index]);
  /** Étape suivante du parcours, pour le lien de fin de page. */
  const suivante = $derived(atelier.etat?.etapes[index + 1]);
  /** Erreur du dernier pilotage. */
  let erreur = $state("");

  /** Leviers SCAMPER : lettre enregistrée dans le champ `levier` et nom de la colonne. */
  const SCAMPER: [string, string][] = [
    ["S", "Substituer"], ["C", "Combiner"], ["A", "Adapter"], ["M", "Modifier"],
    ["P", "Autre usage"], ["E", "Éliminer"], ["R", "Renverser"],
  ];

  /** Termine, rouvre ou saute l'étape, ou change son mode. */
  async function piloter(action: string, mode?: string) {
    erreur = "";
    try {
      await ecrire("piloter", { etape, action, mode });
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }

  /** Cibles d'une sous-étape répétée : la sélection de l'étape si elle porte sur ce type, sinon tous. */
  function cibles(se: SousEtape): string[] {
    if (!e || !se.pour_chaque) return [];
    const sel = e.fiche.sous_etapes.find((s) => s.genre === "selection" && s.parmi === se.pour_chaque);
    if (sel) return e.selections[sel.id] ?? [];
    return deType(se.pour_chaque).filter((a) => a.statut !== "ecarte").map((a) => a.id);
  }
  /** Titre d'un artefact par identifiant. */
  const titre = (id: string) => atelier.etat?.artefacts.find((a) => a.id === id)?.titre ?? "";
  /** Sous-étapes rendues par un écran dédié qui en regroupe plusieurs. */
  const absorbee = (se: SousEtape) => etape === "free-writing" && se.id === "recolte";
  /** Qui agit dans chaque genre de sous-étape, rappelé à côté de son titre. */
  const GENRES: Record<string, string> = { saisie: "à toi", propositions: "le LLM propose", selection: "choix", synthese: "rapport" };
</script>

{#if !e}
  <p class="discret">{atelier.etat ? `Étape inconnue : ${etape}` : "Chargement…"}</p>
{:else}
  <p class="fil"><a href={url.atelier(atelier.parcours)}>Design thinking</a> · {atelier.etat?.tous_parcours.find((p) => p.id === atelier.parcours)?.titre ?? ""} · {PHASES[e.fiche.phase] ?? e.fiche.phase} · étape {index + 1} / {atelier.etat!.etapes.length}</p>
  <h1>{e.fiche.titre}</h1>
  <div class="meta">
    <span class="pastille etat-{e.etat}">{ETATS[e.etat] ?? e.etat}</span>
    <span class="bascule" role="group" aria-label="Mode">
      {#each ["express", "complet"] as m}
        <button class:actif={e.mode === m} onclick={() => piloter("mode", m)}>{m === "express" ? "Express" : "Complet"}</button>
      {/each}
    </span>
    <span class="discret">origine : {e.fiche.origine.join(", ")}</span>
  </div>
  <EnBref fiche={e.fiche} mode={e.mode} />
  {#if e.fiche.regles.length}
    <details class="regles"><summary>Règles de la technique</summary><ul>{#each e.fiche.regles as r}<li>{r}</li>{/each}</ul></details>
  {/if}
  {#if erreur}<p class="erreur">{erreur}</p>{/if}

  {#each e.fiche.sous_etapes.filter((s) => !absorbee(s)) as se, i (se.id)}
    <details class="sous-etape" open>
      <summary><span class="numero">{i + 1}</span> {se.id.replaceAll("_", " ")} <small>{GENRES[se.genre] ?? se.genre}</small></summary>
      {#if etape === "pitch"}
        <Pitch {se} mode={e.mode} />
      {:else if etape === "free-writing" && se.id === "ecriture"}
        <FreeWriting fiche={e.fiche} mode={e.mode} />
      {:else if etape === "bilan" && se.id === "tri"}
        <Tri />
      {:else if se.genre === "selection"}
        <Selection {etape} {se} />
      {:else if se.genre === "synthese"}
        <Synthese {etape} {se} />
      {:else if se.genre === "propositions" && se.pour_chaque}
        {#each cibles(se) as c (c)}
          <section class="cible">
            <h4><a class="id" href={url.document(c)}>{c}</a> {titre(c)}</h4>
            {#if etape === "storyboard" && se.id === "cadre"}<Planche id={c} />{/if}
            <Propositions {etape} {se} cible={c} mode={e.mode} colonnes={etape === "scamper" ? { champ: "levier", valeurs: SCAMPER } : null} />
          </section>
        {:else}
          <p class="discret">Rien à traiter : {se.pour_chaque === "idee" || se.pour_chaque === "besoin" ? "fais d'abord la sélection" : `aucun(e) ${se.pour_chaque} gardé(e)`}.</p>
        {/each}
      {:else if se.genre === "propositions"}
        <Propositions {etape} {se} mode={e.mode} />
      {/if}
    </details>
  {/each}

  {#if etape === "carte-mentale" && terminal.proprietaire}
    {@const carte = deType("carte")[0]}
    {#if carte}
      <h2>Modifier la carte mentale</h2>
      {#key carte.id}<EditeurMermaid id={carte.id} />{/key}
    {/if}
  {/if}

  <div class="actions fin-etape">
    {#if e.etat !== "faite"}
      <button class="principal" onclick={() => piloter("terminer")}>Terminer l'étape</button>
    {:else}
      <button onclick={() => piloter("rouvrir")}>Rouvrir</button>
    {/if}
    {#if e.fiche.optionnelle && e.etat !== "sautee" && e.etat !== "faite"}
      <button onclick={() => piloter("sauter")}>Sauter (optionnelle)</button>
    {/if}
    {#if suivante}<a class="bouton" href={url.etape(suivante.fiche.id, atelier.parcours)}>Étape suivante : {suivante.fiche.titre} →</a>{/if}
  </div>
{/if}
