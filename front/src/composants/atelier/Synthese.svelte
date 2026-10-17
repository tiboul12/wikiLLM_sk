<script lang="ts">
  // Sous-étape `synthese` : assemble le rapport de l'étape à partir des éléments gardés, puis l'affiche.
  import type { SousEtape } from "../../types/SousEtape";
  import { api } from "../../lib/api";
  import { ecrire } from "../../lib/atelier.svelte";
  import Apercu from "./Apercu.svelte";

  let { etape, se }: { etape: string; se: SousEtape } = $props();
  let ids = $state<string[]>([]);
  let version = $state(0);
  let erreur = $state("");
  let enCours = $state(false);

  async function assembler() {
    enCours = true;
    erreur = "";
    try {
      const chemins = await ecrire<string[]>("synthetiser", { etape, sous_etape: se.id });
      const projet = await api.projet();
      ids = chemins.map((c) => projet.documents.find((d) => d.chemin === c)?.id).filter((x): x is string => !!x);
      version++;
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    } finally {
      enCours = false;
    }
  }
</script>

<button class="principal" onclick={assembler} disabled={enCours}>{ids.length ? "Réassembler" : "Assembler"} : {se.cree}</button>
{#if erreur}<p class="erreur">{erreur}</p>{/if}
{#each ids as id (id)}<Apercu {id} {version} />{/each}
