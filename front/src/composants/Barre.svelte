<script lang="ts">
  // Barre latérale : zone Produit (arbre des fichiers) et zone Projet (documents par type).
  // @relu 2026-10-17 4532aeb1
  import type { Projet } from "../types/Projet";
  import type { DocumentResume } from "../types/DocumentResume";
  import { routeur, url, estCourant } from "../lib/routeur.svelte";

  let { projet }: { projet: Projet } = $props();
  let zone = $state<"produit" | "projet">(routeur.route.page === "document" ? "projet" : "produit");

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

  const groupes = $derived.by(() => {
    const m = new Map<string, { rang: number; docs: DocumentResume[] }>();
    for (const d of projet.documents) {
      const [rang, nom] = ORDRE[d.genre] ?? [9, "Autres"];
      if (!m.has(nom)) m.set(nom, { rang, docs: [] });
      m.get(nom)!.docs.push(d);
    }
    return [...m.entries()].sort((a, b) => a[1].rang - b[1].rang);
  });

  const actif = (href: string) => estCourant(href);
</script>

<nav class="onglets" aria-label="Zones">
  <button class:actif={zone === "produit"} onclick={() => (zone = "produit")}>Produit <small>{projet.fichiers.length}</small></button>
  <button class:actif={zone === "projet"} onclick={() => (zone = "projet")}>Projet <small>{projet.documents.length}</small></button>
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
