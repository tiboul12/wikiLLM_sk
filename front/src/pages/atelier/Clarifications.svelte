<script lang="ts">
  // Tableau de bord des clarifications (REC-033, REC-040.2) : un groupe par sujet, avec son état (en cours,
  // clarifié quand toutes ses clarifications sont closes) ; lancer la clarification d'un autre document.
  import type { Projet } from "../../types/Projet";
  import { atelier, champ, sujets } from "../../lib/atelier.svelte";
  import { aller, url, routeur } from "../../lib/routeur.svelte";

  let { projet }: { projet: Projet | null } = $props();
  const titreDoc = (id: string) => projet?.documents.find((d) => d.id === id)?.titre ?? "";
  const groupes = $derived(atelier.etat ? sujets(titreDoc) : []);
  const choisi = $derived((routeur.route, new URLSearchParams(location.search).get("sujet")));
  const candidats = $derived((projet?.documents ?? []).filter((d) => ["besoin", "exigence", "exigence-nf", "storyboard", "idee"].includes(d.genre)));
  let nouveau = $state("");

  $effect(() => {
    if (choisi) document.getElementById("sujet-" + choisi)?.scrollIntoView({ block: "start" });
  });
</script>

<h1>Clarifications</h1>
<p class="discret">
  Revenir sur un besoin, une exigence ou un storyboard quand il est flou : le LLM propose des pistes, tu gardes, corriges ou rejettes. L'agent de code en ouvre aussi pendant le développement (outil <code>atelier_rappel</code>).
</p>

<form class="actions" onsubmit={(e) => { e.preventDefault(); if (nouveau.trim()) aller(url.clarifier(nouveau.trim())); }}>
  <input list="sujets-possibles" bind:value={nouveau} placeholder="B-001, EF-140…" aria-label="Document à clarifier" />
  <datalist id="sujets-possibles">{#each candidats as d (d.id)}<option value={d.id}>{d.titre}</option>{/each}</datalist>
  <button class="principal" type="submit" disabled={!nouveau.trim()}>Clarifier</button>
</form>

{#each groupes as g (g.sujet)}
  <section class="sujet-clarification" class:choisi={g.sujet === choisi} id="sujet-{g.sujet}">
    <h2><a href={url.document(g.sujet)}>{g.sujet}</a> {g.titre} <span class="pastille" class:statut-retenu={g.clarifie}>{g.clarifie ? "clarifié" : "en cours"}</span></h2>
    <ul>
      {#each g.clarifications as c (c.id)}
        <li>
          <a href={url.clarification(c.id)}><span class="id">{c.id}</span> {champ(c, "question")}</a>
          <span class="pastille">{champ(c, "etat") === "close" ? "close" : "en cours"}</span>
          <span class="discret">{((c.frontmatter.pistes as unknown[]) ?? []).length} piste(s) gardée(s)</span>
        </li>
      {/each}
    </ul>
    <a class="bouton" href={url.clarifier(g.sujet)}>{g.clarifie ? "Rouvrir une clarification" : "Reprendre"} →</a>
  </section>
{:else}
  <p class="discret">Aucune clarification pour l'instant.</p>
{/each}
