<script lang="ts">
  // Pastille git d'une correction (EF-165, REC-080, REC-082) : le commit, s'il est poussé, et la version
  // qui l'a publié, une couleur par version ; le sujet du commit au survol.
  import { git, couleurCommit } from "../../lib/reception.svelte";

  let { commit }: { commit: string } = $props();
  /** Couleur et libellé de l'état du commit (publié, poussé, pas poussé, introuvable). */
  const etat = $derived(couleurCommit(commit));
  /** Détail git du commit (sujet, version), chargé avec l'état de la réception. */
  const e = $derived(git.commits[commit]);
</script>

{#if etat}
  <span class="etat-git" style="--teinte: {etat.couleur}" title={e ? `${e.hash.slice(0, 10)} · ${e.date} · ${e.sujet}` : commit}>
    <code>{commit}</code> · {etat.libelle}
  </span>
{/if}

<style>
  .etat-git {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 8px;
    border: 1px solid var(--teinte);
    border-left-width: 4px;
    border-radius: 999px;
    color: var(--teinte);
    font-size: 12px;
    white-space: nowrap;
  }
  .etat-git code {
    color: inherit;
    background: none;
    padding: 0;
  }
</style>
