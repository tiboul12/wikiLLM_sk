<script lang="ts">
  // Accueil du projet (EF-131, EF-140, EF-150) : une tuile par section avec son état, dans l'ordre de la barre
  // (les outils, comme le code, en dernier).
  import type { Projet } from "../types/Projet";
  import { url } from "../lib/routeur.svelte";
  import { SECTIONS } from "../lib/sections.svelte";
  import { atelier } from "../lib/atelier.svelte";
  import { reception, actuelles } from "../lib/reception.svelte";
  import Icone from "../composants/Icone.svelte";

  let { projet }: { projet: Projet | null } = $props();
  /** Part des symboles documentés, pour la tuile du code. */
  const taux = $derived(projet && projet.symboles ? Math.round((100 * projet.documentes) / projet.symboles) : 0);
  /** Nombre de documents d'une zone, et d'un genre si donné. */
  const docs = (zone: string, genre?: string) => (projet?.documents ?? []).filter((d) => d.zone === zone && (!genre || d.genre === genre)).length;
  /** Étapes faites ou sautées du parcours affiché. */
  const etapesFaites = $derived(atelier.etat?.etapes.filter((e) => e.etat === "faite" || e.etat === "sautee").length ?? 0);
  /** Titre de l'étape en cours, s'il y en a une. */
  const enCours = $derived(atelier.etat?.etapes.find((e) => e.etat === "en_cours")?.fiche.titre);
  /** Remarques ouvertes de la réception en cours. */
  const ouvertes = $derived(reception.etat ? actuelles().filter((r) => r.statut === "ouverte").length : 0);
  /** Pages visibles de la carte, hors « Général ». */
  const pages = $derived(reception.etat?.pages.filter((p) => !p.masque && p.cle !== "general").length ?? 0);
</script>

{#if projet}
  <h1>{projet.nom}</h1>
  <p class="discret">Le code, ses documents, la définition du besoin et la réception de l'application, au même endroit.</p>
  <div class="sections-accueil">
    <a class="section-tuile" href={url.projet()}>
      <span class="entete-tuile"><Icone nom="projet" taille={26} /> <strong>Projet</strong></span>
      <span class="chiffre">{docs("projet")} <small>documents</small></span>
      <span class="discret">{docs("projet", "besoin")} besoins · {docs("projet", "exigence")} exigences · {docs("projet", "adr")} décisions</span>
    </a>
    {#if SECTIONS.includes("atelier")}
      <a class="section-tuile" href={url.atelier()}>
        <span class="entete-tuile"><Icone nom="atelier" taille={26} /> <strong>Design thinking</strong></span>
        <span class="chiffre">{etapesFaites} / {atelier.etat?.etapes.length ?? 11} <small>étapes</small></span>
        <span class="discret">{atelier.etat?.projet ? `« ${atelier.etat.projet} »` : "Définir le besoin : commencer par le pitch"}{enCours ? ` · en cours : ${enCours}` : ""}</span>
      </a>
    {/if}
    {#if SECTIONS.includes("reception")}
      <a class="section-tuile" class:attention={ouvertes > 0} href={url.reception()}>
        <span class="entete-tuile"><Icone nom="reception" taille={26} /> <strong>Réception</strong></span>
        <span class="chiffre">{ouvertes} <small>remarque{ouvertes > 1 ? "s" : ""} ouverte{ouvertes > 1 ? "s" : ""}</small></span>
        <span class="discret">{pages} pages trouvées dans le code</span>
      </a>
    {/if}
    <a class="section-tuile" href={url.produit()}>
      <span class="entete-tuile"><Icone nom="produit" taille={26} /> <strong>Code</strong></span>
      <span class="chiffre">{projet.fichiers.length} <small>fichiers</small></span>
      <span class="discret">{projet.symboles} symboles · {taux} % documentés · graphe</span>
      <span class="jauge"><span style="width: {taux}%"></span></span>
    </a>
  </div>
{:else}
  <p class="discret">Chargement…</p>
{/if}
