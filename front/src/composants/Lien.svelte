<script lang="ts">
  // Lien vers un symbole (page du fichier, ancre du symbole) ou vers un document.
  import type { Lien } from "../types/Lien";
  import { url } from "../lib/routeur.svelte";

  let { lien, court = false }: { lien: Lien; court?: boolean } = $props();
  /** Libellé du lien : nom qualifié d'un symbole, « ID · titre » d'un document. */
  const nq = $derived(lien.libelle);
  /** Le lien vise un fichier entier, pas un symbole. */
  const estFichier = $derived(lien.fichier !== null && lien.fichier === lien.id);
  /** Adresse : page du fichier (ancrée sur le symbole s'il y en a un), recherche pour un sujet `topic:`, sinon page du document. */
  const href = $derived(
    estFichier ? url.fichier(lien.id) : lien.fichier ? url.fichier(lien.fichier, nq) : lien.id.startsWith("topic:") ? url.recherche(lien.libelle) : url.document(lien.id),
  );
  /** Texte affiché ; en version courte, le dernier segment du nom d'un symbole. */
  const texte = $derived(estFichier ? lien.id : court && lien.fichier ? nq.split("::").pop() : nq);
</script>

{#if lien.existe}
  <a class="lien" class:probable={lien.confiance === "probable"} {href} title={lien.fichier ? `${lien.fichier}::${nq}` : lien.id}
    >{texte}{#if lien.confiance === "probable"}<sup title="résolu par le nom, sans certitude">?</sup>{/if}</a>
{:else}
  <span class="lien-casse" title="cible introuvable">{texte}</span>
{/if}
