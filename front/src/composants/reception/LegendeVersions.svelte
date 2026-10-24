<script lang="ts">
  // Légende des couleurs de correction (REC-082) : une teinte par version où la correction est faite (tag
  // qui la publie, sinon branche `vN-…`), puis ce que dit la forme de la pastille : publiée, poussée,
  // seulement commitée, introuvable.
  import { git, versionsCitees, couleurVersion } from "../../lib/reception.svelte";

  /** Versions des corrections citées par les remarques, dans l'ordre. */
  const versions = $derived(versionsCitees());
</script>

{#if git.charge && Object.keys(git.commits).length}
  <p class="legende-versions">
    <span class="discret">Corrections :</span>
    {#each versions as v (v)}<span class="pastille-legende publie" style="--teinte: {couleurVersion(v)}">{v}</span>{/each}
    <span class="discret">·</span>
    <span class="pastille-legende publie" style="--teinte: var(--texte-2)">plein</span> publiée (tag)
    <span class="pastille-legende" style="--teinte: var(--texte-2)">contour</span> poussée, pas encore publiée
    <span class="pastille-legende local" style="--teinte: var(--texte-2)">tirets</span> commitée, pas poussée
    <span class="pastille-legende" style="--teinte: var(--rouge)">✗</span> commit introuvable
  </p>
{/if}

<style>
  .legende-versions {
    display: flex;
    gap: 6px 8px;
    flex-wrap: wrap;
    align-items: center;
    font-size: 12.5px;
  }
  .pastille-legende {
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
