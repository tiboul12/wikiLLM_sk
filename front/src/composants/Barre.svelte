<script lang="ts">
  // Barre latérale : zones Produit (arbre des fichiers), Projet, Atelier et Réception (documents par type).
  // @relu 2026-10-17 4532aeb1
  import type { Projet } from "../types/Projet";
  import type { DocumentResume } from "../types/DocumentResume";
  import { routeur, url, estCourant } from "../lib/routeur.svelte";

  let { projet }: { projet: Projet } = $props();
  let zone = $state<"produit" | "projet" | "atelier" | "reception">(routeur.route.page === "document" ? "projet" : "produit");
  const docsProjet = $derived(projet.documents.filter((d) => d.zone === "projet"));
  const docsAtelier = $derived(projet.documents.filter((d) => d.zone === "atelier"));
  const docsReception = $derived(projet.documents.filter((d) => d.zone === "reception"));

  const ORDRE: Record<string, [number, string]> = {
    plan: [0, "Plan"],
    "bdr-analyse": [1, "BDR"],
    "bdr-definition": [1, "BDR"],
    "cahier-des-charges": [2, "Cahier des charges"],
    besoin: [3, "Besoins"],
    exigence: [4, "Exigences"],
    "exigence-nf": [5, "Exigences non fonctionnelles"],
    adr: [6, "Décisions (ADR)"],
    spec: [7, "Specs"],
    // Atelier (SPEC-17), dans l'ordre du parcours.
    parcours: [20, "Parcours"],
    pitch: [21, "Pitch"],
    axe: [22, "Axes"],
    ecrit: [23, "Écrits libres"],
    unite: [24, "Unités de sens"],
    carte: [25, "Carte mentale"],
    probleme: [26, "Problèmes"],
    analogie: [27, "Analogies"],
    idee: [28, "Idées"],
    analyse: [29, "Analyses"],
    scamper: [30, "SCAMPER"],
    storyboard: [31, "Storyboards"],
    clarification: [32, "Clarifications"],
    // Cahier de réception (SPEC-23).
    "carte-reception": [40, "Carte des pages"],
    remarque: [41, "Remarques"],
  };

  const dossiers = $derived.by(() => {
    const m = new Map<string, typeof projet.fichiers>();
    for (const f of projet.fichiers) {
      const d = f.chemin.includes("/") ? f.chemin.slice(0, f.chemin.lastIndexOf("/")) : ".";
      if (!m.has(d)) m.set(d, []);
      m.get(d)!.push(f);
    }
    return [...m.entries()];
  });

  const grouper = (docs: DocumentResume[]) => {
    const m = new Map<string, { rang: number; docs: DocumentResume[] }>();
    for (const d of docs) {
      const [rang, nom] = ORDRE[d.genre] ?? [9, "Autres"];
      if (!m.has(nom)) m.set(nom, { rang, docs: [] });
      m.get(nom)!.docs.push(d);
    }
    return [...m.entries()].sort((a, b) => a[1].rang - b[1].rang);
  };
  const groupes = $derived(grouper(zone === "atelier" ? docsAtelier : zone === "reception" ? docsReception : docsProjet));

  const actif = (href: string) => estCourant(href);
</script>

<nav class="onglets" aria-label="Zones">
  <button class:actif={zone === "produit"} onclick={() => (zone = "produit")}>Produit <small>{projet.fichiers.length}</small></button>
  <button class:actif={zone === "projet"} onclick={() => (zone = "projet")}>Projet <small>{docsProjet.length}</small></button>
  {#if docsAtelier.length}
    <button class:actif={zone === "atelier"} onclick={() => (zone = "atelier")}>Atelier <small>{docsAtelier.length}</small></button>
  {/if}
  {#if docsReception.length}
    <button class:actif={zone === "reception"} onclick={() => (zone = "reception")} title="Cahier de réception">Récep. <small>{docsReception.length}</small></button>
  {/if}
</nav>

{#key routeur.route}
  {#if zone === "produit"}
    {#each dossiers as [dossier, fichiers]}
      <details open>
        <summary>{dossier}/</summary>
        <ul>
          {#each fichiers as f}
            {@const href = url.fichier(f.chemin)}
            <li>
              <a {href} class:actif={actif(href)} title={f.role ?? ""}>
                {f.chemin.split("/").pop()}
                <span class="compteur" class:faible={f.symboles > 0 && f.documentes / f.symboles < 0.5}>{f.documentes}/{f.symboles}</span>
              </a>
            </li>
          {/each}
        </ul>
      </details>
    {/each}
  {:else}
    {#each groupes as [nom, g]}
      <details open={g.docs.length <= 12}>
        <summary>{nom} <small>{g.docs.length}</small></summary>
        <ul>
          {#each g.docs as d}
            {@const href = url.document(d.id)}
            <li>
              <a {href} class:actif={actif(href)} title={d.titre}>
                <span class="id">{d.id}</span> {d.titre === d.id ? "" : d.titre}
              </a>
            </li>
          {/each}
        </ul>
      </details>
    {/each}
  {/if}
{/key}
