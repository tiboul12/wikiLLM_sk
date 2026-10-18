<script lang="ts">
  // Mise en page : en-tête (projet, recherche), barre latérale (zones Produit, Projet, Atelier), page courante.
  // Servie par `wikik atelier`, l'application ajoute les écrans de l'atelier (SPEC-20) ; par
  // `wikik reception`, ceux du cahier de réception (SPEC-23).
  // @relu 2026-10-17 a957d8e5
  import { api } from "./lib/api";
  import { routeur, aller, url } from "./lib/routeur.svelte";
  import { projetCourant } from "./lib/hook";
  import type { Projet } from "./types/Projet";
  import Barre from "./composants/Barre.svelte";
  import Accueil from "./pages/Accueil.svelte";
  import PageFichier from "./pages/PageFichier.svelte";
  import PageDocument from "./pages/PageDocument.svelte";
  import Recherche from "./pages/Recherche.svelte";
  import PageGraphe from "./pages/PageGraphe.svelte";
  import Tableau from "./pages/atelier/Tableau.svelte";
  import Etape from "./pages/atelier/Etape.svelte";
  import Clarification from "./pages/atelier/Clarification.svelte";
  import Clarifier from "./pages/atelier/Clarifier.svelte";
  import { atelier, detecter } from "./lib/atelier.svelte";
  import { reception, detecterReception } from "./lib/reception.svelte";
  import CarteReception from "./pages/reception/Carte.svelte";
  import PageReception from "./pages/reception/PageReception.svelte";
  import Traitement from "./pages/reception/Traitement.svelte";

  let projet = $state<Projet | null>(null);
  let erreur = $state("");
  let saisie = $state("");
  let barreOuverte = $state(false);

  api
    .projet()
    .then((p) => {
      projet = p;
      projetCourant.nom = p.nom;
      atelier.url = p.url_atelier;
      document.title = `${p.nom} · wikik`;
    })
    .catch((e) => (erreur = String(e)));
  detecter().then((oui) => {
    if (oui && projet) document.title = `${projet.nom} · atelier wikik`;
  });
  detecterReception().then((oui) => {
    if (oui) document.title = `${projet?.nom ?? "projet"} · réception wikik`;
  });

  // Durée de l'appel au LLM en cours, pour le bandeau.
  let maintenant = $state(Date.now());
  $effect(() => {
    if (!atelier.occupe) return;
    const t = setInterval(() => (maintenant = Date.now()), 500);
    return () => clearInterval(t);
  });

  function chercher(e: SubmitEvent) {
    e.preventDefault();
    if (saisie.trim()) aller(url.recherche(saisie.trim()));
  }

  $effect(() => {
    routeur.route;
    barreOuverte = false;
  });
</script>

<header class="entete">
  <button class="menu" aria-label="Afficher la navigation" onclick={() => (barreOuverte = !barreOuverte)}>☰</button>
  <a class="marque" href={url.accueil()}><span class="logo">w</span>{projet?.nom ?? "wikik"}</a>
  <form class="recherche" onsubmit={chercher} role="search">
    <input type="search" placeholder="Rechercher dans la doc…" bind:value={saisie} aria-label="Rechercher" />
  </form>
  {#if atelier.actif}<a class="lien-atelier" href={url.atelier()}>Atelier</a>{/if}
  {#if reception.actif}<a class="lien-atelier" href={url.reception()}>Réception</a>{/if}
  <a class="lien-graphe" href={url.graphe()}>Graphe</a>
</header>
{#if atelier.occupe}
  <div class="bandeau-llm" role="status">
    Le LLM réfléchit ({Math.round((maintenant - atelier.depuis) / 1000)} s) : il lit le projet par MCP et prépare ses propositions.
  </div>
{:else if reception.occupe}
  <div class="bandeau-llm" role="status">Le LLM lit le projet par MCP et cherche les écrans que le scan n'a pas trouvés…</div>
{/if}

<div class="cadre">
  <aside class="barre" class:ouverte={barreOuverte}>
    {#if projet}
      <Barre {projet} />
    {:else if erreur}
      <p class="erreur">{erreur}</p>
    {:else}
      <p class="discret">Chargement…</p>
    {/if}
  </aside>
  <main class="contenu">
    {#if routeur.route.page === "fichier"}
      {#key routeur.route.chemin}<PageFichier chemin={routeur.route.chemin} ancre={routeur.route.ancre} />{/key}
    {:else if routeur.route.page === "document"}
      {#key routeur.route.id}<PageDocument id={routeur.route.id} ancre={routeur.route.ancre} />{/key}
    {:else if routeur.route.page === "recherche"}
      {#key routeur.route.q}<Recherche q={routeur.route.q} />{/key}
    {:else if routeur.route.page === "graphe"}
      {#key routeur.route.centre}<PageGraphe centre={routeur.route.centre} />{/key}
    {:else if atelier.actif && atelier.etat && routeur.route.page === "etape"}
      {#key routeur.route.etape}<Etape etape={routeur.route.etape} />{/key}
    {:else if reception.actif && reception.etat && routeur.route.page === "reception-page"}
      {#key routeur.route.cle}<PageReception cle={routeur.route.cle} />{/key}
    {:else if reception.actif && reception.etat && routeur.route.page === "traitement"}
      <Traitement />
    {:else if reception.actif && reception.etat && (routeur.route.page === "reception" || routeur.route.page === "accueil")}
      <CarteReception />
    {:else if atelier.actif && atelier.etat && routeur.route.page === "clarifier"}
      {#key routeur.route.sujet}<Clarifier sujet={routeur.route.sujet} />{/key}
    {:else if atelier.actif && atelier.etat && routeur.route.page === "clarification"}
      {#key routeur.route.id}<Clarification id={routeur.route.id} />{/key}
    {:else if atelier.actif && atelier.etat && (routeur.route.page === "atelier" || routeur.route.page === "accueil")}
      <Tableau />
    {:else if routeur.route.page !== "accueil"}
      <p class="discret">{atelier.actif || reception.actif ? "Chargement…" : "Cette page n'existe que dans l'atelier (wikik atelier) ou le cahier de réception (wikik reception)."}</p>
    {:else}
      <Accueil {projet} />
    {/if}
  </main>
</div>
