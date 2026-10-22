<script lang="ts">
  // Éditeur d'un document (EF-158, ADR-0016) : le fichier entier (frontmatter compris) à gauche, l'aperçu
  // à droite ; « Enregistrer » (Ctrl+S) remplace le fichier, Ctrl+Z hors du champ annule l'enregistrement.
  import { onMount } from "svelte";
  import { sourceDocument, ecrireDocument } from "../lib/edition.svelte";
  import { rendre } from "../lib/markdown";
  import { autoLien } from "../lib/connus.svelte";

  let { id, fermer }: { id: string; fermer: () => void } = $props();
  let chemin = $state("");
  let texte = $state("");
  let origine = $state("");
  let erreur = $state("");
  let enCours = $state(false);

  onMount(() => {
    sourceDocument(id)
      .then((s) => {
        chemin = s.chemin;
        texte = origine = s.texte;
      })
      .catch((e) => (erreur = String(e instanceof Error ? e.message : e)));
  });

  const corps = $derived(texte.replace(/^---\n[\s\S]*?\n---\n?/, ""));
  const apercu = $derived(rendre(corps, autoLien, undefined, chemin, autoLien));
  const modifie = $derived(texte !== origine);

  async function enregistrer() {
    if (!modifie || enCours) return;
    enCours = true;
    erreur = "";
    try {
      await ecrireDocument(id, texte);
      origine = texte;
      fermer();
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    } finally {
      enCours = false;
    }
  }
  function touche(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      enregistrer();
    }
  }
  function quitter() {
    if (modifie && !confirm("Abandonner les modifications non enregistrées ?")) return;
    fermer();
  }
</script>

<section class="editeur">
  <div class="barre-editeur">
    <strong>Édition de {id}</strong> <code class="discret">{chemin}</code>
    <span class="espace"></span>
    {#if erreur}<span class="erreur">{erreur}</span>{/if}
    <button onclick={quitter}>Fermer</button>
    <button class="principal" onclick={enregistrer} disabled={!modifie || enCours} title="Ctrl+S">{enCours ? "Enregistrement…" : "Enregistrer"}</button>
  </div>
  <div class="volets">
    <textarea class="mono" bind:value={texte} onkeydown={touche} spellcheck="true" aria-label="Texte du document"></textarea>
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    <article class="prose apercu">{@html apercu}</article>
  </div>
</section>

<style>
  .editeur {
    border: 1px solid var(--bord-fort);
    border-radius: var(--rayon);
    margin: 12px 0 20px;
    background: var(--fond);
  }
  .barre-editeur {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    padding: 8px 12px;
    border-bottom: 1px solid var(--bord);
    background: var(--fond-2);
  }
  .espace {
    flex: 1;
  }
  .volets {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    min-height: 60vh;
  }
  textarea {
    border: none;
    border-right: 1px solid var(--bord);
    border-radius: 0;
    padding: 12px;
    resize: vertical;
    min-height: 60vh;
    font-size: 13px;
    line-height: 1.5;
    background: var(--fond);
    color: var(--texte);
  }
  .apercu {
    padding: 4px 16px;
    overflow: auto;
    max-height: 75vh;
  }
  @media (max-width: 800px) {
    .volets {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
