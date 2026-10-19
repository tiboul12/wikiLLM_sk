<script lang="ts">
  // Mise en page (EF-131, EF-140) : barre d'activité à gauche (sections), panneau de la section active
  // (largeur réglable en tirant son bord), page courante ; en-tête avec le projet et la recherche. Le
  // parcours du Design thinking affiché suit l'URL ; le bandeau d'un appel au LLM permet de l'annuler ;
  // le bouton Terminal (propriétaire seulement, ADR-0013) ouvre le volet droit du terminal. Le serveur du projet sert les quatre sections
  // (ADR-0012) ; le site exporté n'a que l'accueil, Produit et Projet.
  // @relu 2026-10-19 64b2a015
  import { api } from "./lib/api";
  import { routeur, aller, url, parcoursDeUrl } from "./lib/routeur.svelte";
  import { projetCourant } from "./lib/hook";
  import { retenir } from "./lib/connus.svelte";
  import type { Projet } from "./types/Projet";
  import { sectionDe, navigation } from "./lib/sections.svelte";
  import BarreActivite from "./composants/BarreActivite.svelte";
  import Barre from "./composants/Barre.svelte";
  import Accueil from "./pages/Accueil.svelte";
  import ProduitAccueil from "./pages/ProduitAccueil.svelte";
  import ProjetAccueil from "./pages/ProjetAccueil.svelte";
  import Relecture from "./pages/Relecture.svelte";
  import PageFichier from "./pages/PageFichier.svelte";
  import PageDocument from "./pages/PageDocument.svelte";
  import Recherche from "./pages/Recherche.svelte";
  import PageGraphe from "./pages/PageGraphe.svelte";
  import DesignThinking from "./pages/atelier/DesignThinking.svelte";
  import Etape from "./pages/atelier/Etape.svelte";
  import Clarification from "./pages/atelier/Clarification.svelte";
  import Clarifier from "./pages/atelier/Clarifier.svelte";
  import Clarifications from "./pages/atelier/Clarifications.svelte";
  import { atelier, detecter, choisirParcours, annulerLLM } from "./lib/atelier.svelte";
  import { reception, detecterReception } from "./lib/reception.svelte";
  import CahierReception from "./pages/reception/CahierReception.svelte";
  import CarteReception from "./pages/reception/CarteReception.svelte";
  import Traitement from "./pages/reception/Traitement.svelte";
  import Parametres from "./pages/Parametres.svelte";
  import PanneauTerminal from "./composants/PanneauTerminal.svelte";
  import Icone from "./composants/Icone.svelte";
  import { terminal, chargerParametres } from "./lib/terminal.svelte";

  let projet = $state<Projet | null>(null);
  let erreur = $state("");
  let saisie = $state("");
  let barreOuverte = $state(false);

  api
    .projet()
    .then((p) => {
      projet = p;
      projetCourant.nom = p.nom;
      retenir(p);
      document.title = `${p.nom} · wikik`;
    })
    .catch((e) => (erreur = String(e)));
  detecter().then((ok) => ok && choisirParcours(parcoursDeUrl()));
  detecterReception();
  chargerParametres();

  // Le parcours affiché suit l'URL des pages du Design thinking (EF-145).
  $effect(() => {
    const p = routeur.route.page;
    if (atelier.actif && (p === "atelier" || p === "etape")) choisirParcours(parcoursDeUrl());
  });

  // Section active : celle de la page, ou la dernière ouverte pour une page commune (recherche, graphe).
  const section = $derived(sectionDe(routeur.route, projet) ?? navigation.section);
  $effect(() => {
    navigation.section = section;
  });
  const avecPanneau = $derived(section !== "accueil" && section !== "parametres");

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

  // Largeur du panneau, réglée en tirant son bord (REC-011) et gardée par ce navigateur.
  const lireLargeur = () => {
    try {
      return Number(localStorage.getItem("wikik.panneau")) || 280;
    } catch {
      return 280;
    }
  };
  let largeur = $state(lireLargeur());
  const lireLargeurTerminal = () => {
    try {
      return Number(localStorage.getItem("wikik.terminal")) || 560;
    } catch {
      return 560;
    }
  };
  let largeurTerminal = $state(lireLargeurTerminal());
  function tirer(e: PointerEvent) {
    const poignee = e.currentTarget as HTMLElement;
    poignee.setPointerCapture(e.pointerId);
    const bouger = (m: PointerEvent) => (largeur = Math.min(700, Math.max(180, m.clientX - 56)));
    const lacher = () => {
      poignee.removeEventListener("pointermove", bouger);
      try {
        localStorage.setItem("wikik.panneau", String(largeur));
      } catch {
        /* sans stockage, la largeur vaut pour la session */
      }
    };
    poignee.addEventListener("pointermove", bouger);
    poignee.addEventListener("pointerup", lacher, { once: true });
  }
</script>

<header class="entete">
  {#if avecPanneau}<button class="menu" aria-label="Afficher la navigation" onclick={() => (barreOuverte = !barreOuverte)}>☰</button>{/if}
  <a class="marque" href={url.accueil()}><span class="logo">w</span>{projet?.nom ?? "wikik"}</a>
  <form class="recherche" onsubmit={chercher} role="search">
    <input type="search" placeholder="Rechercher dans la doc…" bind:value={saisie} aria-label="Rechercher" />
  </form>
  {#if terminal.proprietaire}
    <button class="bouton-terminal" class:actif={terminal.ouvert} onclick={() => (terminal.ouvert = !terminal.ouvert)} title="Terminal du projet (ton agent de développement)" aria-pressed={terminal.ouvert}>
      <Icone nom="terminal" taille={18} /> <span>Terminal</span>
    </button>
  {/if}
</header>
{#if atelier.occupe}
  <div class="bandeau-llm" role="status">
    Le LLM réfléchit ({Math.round((maintenant - atelier.depuis) / 1000)} s) : il lit le projet par MCP et prépare ses propositions.
    <button class="annuler-llm" onclick={annulerLLM}>Annuler</button>
  </div>
{:else if reception.occupe}
  <div class="bandeau-llm" role="status">
    Le LLM lit le projet par MCP et cherche les écrans que le scan n'a pas trouvés…
    <button class="annuler-llm" onclick={annulerLLM}>Annuler</button>
  </div>
{/if}

<div class="cadre" class:sans-panneau={!avecPanneau} class:avec-terminal={terminal.ouvert} style="--largeur-panneau: {largeur}px; --largeur-terminal: {largeurTerminal}px">
  <BarreActivite active={routeur.route.page === "graphe" ? null : section} graphe={routeur.route.page === "graphe"} />
  {#if avecPanneau}
    <aside class="barre" class:ouverte={barreOuverte}>
      {#if projet}
        <Barre {projet} {section} />
      {:else if erreur}
        <p class="erreur">{erreur}</p>
      {:else}
        <p class="discret">Chargement…</p>
      {/if}
    </aside>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="poignee" onpointerdown={tirer} ondblclick={() => (largeur = 280)} title="Tirer pour élargir le panneau (double-clic : largeur d'origine)"></div>
  {/if}
  <main class="contenu">
    {#if routeur.route.page === "fichier"}
      {#key routeur.route.chemin}<PageFichier chemin={routeur.route.chemin} ancre={routeur.route.ancre} />{/key}
    {:else if routeur.route.page === "document"}
      {#key routeur.route.id}<PageDocument id={routeur.route.id} ancre={routeur.route.ancre} />{/key}
    {:else if routeur.route.page === "recherche"}
      {#key routeur.route.q}<Recherche q={routeur.route.q} />{/key}
    {:else if routeur.route.page === "graphe"}
      {#key routeur.route.centre}<PageGraphe centre={routeur.route.centre} />{/key}
    {:else if routeur.route.page === "produit"}
      <ProduitAccueil {projet} />
    {:else if routeur.route.page === "projet"}
      <ProjetAccueil {projet} />
    {:else if routeur.route.page === "relecture"}
      <Relecture {projet} />
    {:else if routeur.route.page === "accueil"}
      <Accueil {projet} />
    {:else if routeur.route.page === "parametres"}
      <Parametres />
    {:else if !atelier.actif && !reception.actif}
      <p class="discret">Cette page n'existe pas dans le site exporté : lance <code>wikik</code> dans le projet.</p>
    {:else if routeur.route.page === "etape"}
      {#if atelier.etat}{#key routeur.route.etape}<Etape etape={routeur.route.etape} />{/key}{:else}<p class="discret">Chargement…</p>{/if}
    {:else if routeur.route.page === "clarification"}
      {#if atelier.etat}{#key routeur.route.id}<Clarification id={routeur.route.id} />{/key}{:else}<p class="discret">Chargement…</p>{/if}
    {:else if routeur.route.page === "clarifications"}
      {#if atelier.etat}<Clarifications {projet} />{:else}<p class="discret">Chargement…</p>{/if}
    {:else if routeur.route.page === "clarifier"}
      {#if atelier.etat}{#key routeur.route.sujet}<Clarifier sujet={routeur.route.sujet} />{/key}{:else}<p class="discret">Chargement…</p>{/if}
    {:else if routeur.route.page === "atelier"}
      {#if atelier.etat}<DesignThinking />{:else}<p class="discret">Chargement…</p>{/if}
    {:else if routeur.route.page === "reception-page"}
      {#if reception.etat}{#key routeur.route.cle}<CarteReception cle={routeur.route.cle} />{/key}{:else}<p class="discret">Chargement…</p>{/if}
    {:else if routeur.route.page === "traitement"}
      {#if reception.etat}<Traitement />{:else}<p class="discret">Chargement…</p>{/if}
    {:else if routeur.route.page === "reception"}
      {#if reception.etat}<CahierReception />{:else}<p class="discret">Chargement…</p>{/if}
    {/if}
  </main>
</div>
{#if terminal.ouvert}<PanneauTerminal bind:largeur={largeurTerminal} />{/if}
