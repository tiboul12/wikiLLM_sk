<script lang="ts">
  // Pastille de version d'une correction (REC-082) : « v8 » dans la teinte de sa version ; pleine si la
  // version est publiée (tag), en contour si elle est poussée sans être publiée, en tirets si elle n'est que
  // commitée, rouge si le commit est introuvable. Le détail est au survol.
  import { couleurCommit } from "../../lib/reception.svelte";

  let { commit }: { commit: string | null | undefined } = $props();
  /** Version, couleur et état de la correction ; rien sans commit. */
  const c = $derived(couleurCommit(commit));
</script>

{#if c}
  <span class="pastille-version {c.etat}" style="--teinte: {c.couleur}" title="{commit} · {c.libelle}">{c.etat === "introuvable" ? "✗" : (c.version ?? "?")}</span>
{/if}

<style>
  .pastille-version {
    display: inline-block;
    flex: none;
    margin-left: auto;
    padding: 0 6px;
    border: 1.5px solid var(--teinte);
    border-radius: 999px;
    color: var(--teinte);
    font-size: 11px;
    font-weight: 650;
    line-height: 16px;
    font-family: var(--mono);
  }
  .publie {
    background: var(--teinte);
    color: var(--fond);
  }
  .local {
    border-style: dashed;
  }
</style>
