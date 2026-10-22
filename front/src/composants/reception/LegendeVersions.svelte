<script lang="ts">
  // Légende des couleurs de correction (REC-082) : une teinte par version publiée, puis « poussé, pas
  // publié », « pas poussé », « introuvable ».
  import { git, versionsCitees, couleurCommit } from "../../lib/reception.svelte";

  /** Versions publiées citées par les remarques, dans l'ordre de leurs teintes. */
  const versions = $derived(versionsCitees());
  /** Couleur d'une version, prise sur l'un de ses commits pour rester identique à celle des pastilles. */
  const echantillon = (v: string) => {
    const e = Object.values(git.commits).find((x) => x.version === v);
    return e ? couleurCommit(e.commit)?.couleur : "var(--gris)";
  };
</script>

{#if git.charge && Object.keys(git.commits).length}
  <p class="legende-versions">
    <span class="discret">Corrections :</span>
    {#each versions as v (v)}<span class="cle" style="--teinte: {echantillon(v)}">publiées en {v}</span>{/each}
    <span class="cle" style="--teinte: var(--accent)">poussées, pas encore publiées</span>
    <span class="cle" style="--teinte: var(--orange)">commitées, pas poussées</span>
    <span class="cle" style="--teinte: var(--rouge)">commit introuvable</span>
  </p>
{/if}

<style>
  .legende-versions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    align-items: center;
    font-size: 12.5px;
  }
  .cle::before {
    content: "";
    display: inline-block;
    width: 10px;
    height: 10px;
    margin-right: 5px;
    border-radius: 3px;
    background: var(--teinte);
    vertical-align: -1px;
  }
</style>
