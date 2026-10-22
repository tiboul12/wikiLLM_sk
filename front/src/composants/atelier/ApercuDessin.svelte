<script lang="ts">
  // Aperçu d'un dessin de l'atelier (EF-167) : le SVG enregistré, assaini avant d'être affiché.
  import DOMPurify from "dompurify";

  let { nom, version = 0 }: { nom: string; version?: number } = $props();
  let html = $state("");
  $effect(() => {
    version;
    fetch(`/api/dessin/lire?nom=${encodeURIComponent(nom)}`)
      .then((r) => (r.ok ? r.json() : null))
      .then((t: string | null) => (html = t ? DOMPurify.sanitize(t, { USE_PROFILES: { svg: true, svgFilters: true } }) : ""))
      .catch(() => (html = ""));
  });
</script>

{#if html}
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  <div class="apercu-dessin">{@html html}</div>
{/if}

<style>
  .apercu-dessin :global(svg) {
    width: 100%;
    height: auto;
    border: 1px solid var(--bord);
    border-radius: 6px;
    background: white;
  }
</style>
