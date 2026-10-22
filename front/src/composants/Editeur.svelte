<script lang="ts">
  // Éditeur d'un document (EF-158, ADR-0016) : le fichier entier (frontmatter compris) à gauche, l'aperçu
  // à droite ; « Enregistrer » (Ctrl+S) remplace le fichier, Ctrl+Z hors du champ annule l'enregistrement.
  import { onMount } from "svelte";
  import { sourceDocument, ecrireDocument } from "../lib/edition.svelte";
  import { rendre } from "../lib/markdown";
  import { autoLien } from "../lib/connus.svelte";

  let { id, fermer }: { id: string; fermer: () => void } = $props();
  /** Chemin du fichier édité, pour résoudre les images relatives de l'aperçu. */
  let chemin = $state("");
  /** Texte en cours d'édition, frontmatter compris. */
  let texte = $state("");
  /** Texte tel qu'il est sur le disque, pour savoir s'il y a des modifications à perdre. */
  let origine = $state("");
  /** Erreur de lecture ou d'enregistrement. */
  let erreur = $state("");
  /** Enregistrement en cours : un second Ctrl+S est ignoré. */
  let enCours = $state(false);

  onMount(() => {
    sourceDocument(id)
      .then((s) => {
        chemin = s.chemin;
        texte = origine = s.texte;
      })
      .catch((e) => (erreur = String(e instanceof Error ? e.message : e)));
  });

  /** Corps sans le frontmatter, seul rendu dans l'aperçu. */
  const corps = $derived(texte.replace(/^---\n[\s\S]*?\n---\n?/, ""));
  /** Aperçu HTML du corps, identifiants cités liés comme sur la page du document. */
  const apercu = $derived(rendre(corps, autoLien, undefined, chemin, autoLien));
  /** Le texte diffère du fichier. */
  const modifie = $derived(texte !== origine);

  /** Remplace le fichier par le texte édité puis ferme l'éditeur ; rien n'est envoyé sans modification. */
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
  /** Ctrl+S (ou Cmd+S) enregistre au lieu d'ouvrir l'enregistrement du navigateur. */
  function touche(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      enregistrer();
    }
  }
  /** Ferme l'éditeur, après confirmation s'il reste des modifications. */
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
