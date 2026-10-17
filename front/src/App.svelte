<script lang="ts">
  // Mise en page : en-tête (projet, recherche), barre latérale (zones Produit et Projet), page courante.
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

  let projet = $state<Projet | null>(null);
  let erreur = $state("");
  let saisie = $state("");
  let barreOuverte = $state(false);

  api
    .projet()
    .then((p) => {
      projet = p;
      projetCourant.nom = p.nom;
      document.title = `${p.nom} · wikik`;
    })
    .catch((e) => (erreur = String(e)));

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
  <a class="marque" href="/"><span class="logo">w</span>{projet?.nom ?? "wikik"}</a>
  <form class="recherche" onsubmit={chercher} role="search">
    <input type="search" placeholder="Rechercher dans la doc…" bind:value={saisie} aria-label="Rechercher" />
  </form>
  <a class="lien-graphe" href={url.graphe()}>Graphe</a>
</header>

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
    {:else}
      <Accueil {projet} />
    {/if}
  </main>
</div>
