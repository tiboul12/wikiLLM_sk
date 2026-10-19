<script lang="ts">
  // Panneau latéral de la section active (EF-131) : arbre des fichiers (Produit), documents par type
  // (Projet), étapes et artefacts (Atelier), pages et remarques (Réception).
  import type { Projet } from "../types/Projet";
  import type { DocumentResume } from "../types/DocumentResume";
  import { routeur, url, estCourant } from "../lib/routeur.svelte";
  import type { Section } from "../lib/sections.svelte";
  import { atelier, ETATS } from "../lib/atelier.svelte";
  import { reception, remarquesDe } from "../lib/reception.svelte";

  let { projet, section }: { projet: Projet; section: Section } = $props();
  const docsDe = (zone: string) => projet.documents.filter((d) => d.zone === zone);

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
  const groupes = $derived(grouper(docsDe(section === "atelier" ? "atelier" : section === "reception" ? "reception" : "projet")));
  const TITRES: Record<string, string> = { produit: "Produit", projet: "Projet", atelier: "Atelier", reception: "Réception" };

  const actif = (href: string) => estCourant(href);
</script>

<p class="titre-panneau">{TITRES[section] ?? ""}</p>

{#key routeur.route}
  {#if section === "produit"}
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
    {#if section === "atelier" && atelier.etat}
      <details open>
        <summary>Parcours</summary>
        <ul>
          {#each atelier.etat.etapes as e, i (e.fiche.id)}
            {@const href = url.etape(e.fiche.id)}
            <li>
              <a {href} class:actif={actif(href)} title={ETATS[e.etat] ?? e.etat}>
                <span class="puce etat-{e.etat}">{e.etat === "faite" ? "✓" : i + 1}</span> {e.fiche.titre}
              </a>
            </li>
          {/each}
        </ul>
      </details>
    {/if}
    {#if section === "reception" && reception.etat}
      <ul class="liens-section">
        <li><a href={url.traitement()} class:actif={actif(url.traitement())}>Traiter les remarques</a></li>
      </ul>
      <details open>
        <summary>Pages <small>{reception.etat.pages.filter((p) => !p.masque).length}</small></summary>
        <ul>
          {#each reception.etat.pages.filter((p) => !p.masque) as p (p.cle)}
            {@const href = url.pageReception(p.cle)}
            {@const n = remarquesDe(p.cle).filter((r) => r.statut === "ouverte").length}
            <li>
              <a {href} class:actif={actif(href) && new URLSearchParams(location.search).get("cle") === p.cle} title={p.route ?? p.nom}>
                {p.nom}
                {#if n}<span class="compteur faible">{n}</span>{/if}
              </a>
            </li>
          {/each}
        </ul>
      </details>
    {/if}
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
    {:else}
      {#if section === "projet"}<p class="discret">Aucun document : <code>wikik nouveau besoin "…"</code>, ou l'atelier.</p>{/if}
    {/each}
  {/if}
{/key}
