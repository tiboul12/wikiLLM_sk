<script lang="ts">
  // Sous-étape `selection` : les artefacts d'un type à cocher (idées à analyser, besoins à décliner).
  import type { SousEtape } from "../../types/SousEtape";
  import { atelier, deType, ecrire } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";

  let { etape, se }: { etape: string; se: SousEtape } = $props();
  /** Artefacts déjà cochés pour cette sous-étape. */
  const choix = $derived(atelier.etat?.etapes.find((e) => e.fiche.id === etape)?.selections[se.id] ?? []);
  /** Artefacts du type demandé, hors écartés. */
  const candidats = $derived(deType(se.parmi).filter((a) => a.statut !== "ecarte"));
  /** Erreur d'enregistrement de la sélection. */
  let erreur = $state("");

  /** Coche ou décoche un artefact et enregistre aussitôt la sélection entière. */
  async function basculer(id: string) {
    const ids = choix.includes(id) ? choix.filter((x) => x !== id) : [...choix, id];
    erreur = "";
    try {
      await ecrire("selection", { etape, sous_etape: se.id, ids });
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }
</script>

{#if !candidats.length}
  <p class="discret">Aucun(e) {se.parmi} pour l'instant : les étapes précédentes en produisent.</p>
{:else}
  <ul class="selection">
    {#each candidats as a (a.id)}
      <li>
        <label>
          <input type="checkbox" checked={choix.includes(a.id)} onchange={() => basculer(a.id)} />
          <a class="id" href={url.document(a.id)}>{a.id}</a> {a.titre}
        </label>
      </li>
    {/each}
  </ul>
  <p class="discret">{choix.length} sélectionné(s){se.preselection ? ` · présélection : ${se.preselection}` : ""}</p>
{/if}
{#if erreur}<p class="erreur">{erreur}</p>{/if}
