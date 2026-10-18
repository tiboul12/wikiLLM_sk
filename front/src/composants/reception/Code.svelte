<script lang="ts">
  // Lien vers le code d'un symbole (`chemin::Nom`) dans le wiki : le fichier, ouvert sur sa cellule.
  import { url } from "../../lib/routeur.svelte";

  let { id }: { id: string } = $props();
  const i = $derived(id.indexOf("::"));
  const fichier = $derived(i < 0 ? id : id.slice(0, i));
  const nom = $derived(i < 0 ? "" : id.slice(i + 2));
</script>

<a class="code-lien" href={url.fichier(fichier, nom || undefined)} title={id}><code>{nom || fichier}</code> <span class="discret">{fichier}</span></a>
