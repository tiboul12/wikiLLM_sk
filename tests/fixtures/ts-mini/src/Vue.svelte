<!-- @component
Affiche une valeur doublée.
@req EF-001
-->
<script lang="ts">
  // Vue de démonstration (fixture Svelte).
  import { doubler } from "./lib/calcul";

  let { valeur }: { valeur: number } = $props();

  /** Valeur affichée, doublée. */
  function affichee(): number {
    return doubler(valeur);
  }
</script>

<p>{affichee()}</p>
