<script lang="ts">
  // Aperçu d'un document produit par l'atelier (carte mentale, rapport, BDR…), rendu comme dans le wiki.
  // @relu 2026-10-18 6afdbaff
  import { tick } from "svelte";
  import { api } from "../../lib/api";
  import { rendre, rendreMermaid } from "../../lib/markdown";
  import { url } from "../../lib/routeur.svelte";

  let { id, version = 0 }: { id: string; version?: number } = $props();
  /** Document rendu en HTML, liens résolus par l'index. */
  let html = $state("");
  /** Erreur de chargement du document. */
  let erreur = $state("");
  /** Conteneur du rendu, où l'on dessine les diagrammes Mermaid. */
  let corps = $state<HTMLElement | null>(null);

  $effect(() => {
    version;
    api
      .document(id)
      .then((p) => {
        html = rendre(p.corps, (b) => {
          const l = p.liens.find((x) => x.brute === b)?.cible;
          return l ? (l.fichier ? url.fichier(l.fichier, l.libelle) : url.document(l.id)) : null;
        }, undefined, p.chemin);
        tick().then(() => corps && rendreMermaid(corps));
      })
      .catch((e) => (erreur = String(e)));
  });
</script>

<div class="apercu">
  <p class="discret"><a href={url.document(id)}>{id}</a> dans le wiki</p>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
  <article class="prose" bind:this={corps}>{@html html}</article>
</div>
