<script lang="ts">
  // Éditeur Mermaid (EF-167, REC-078) : le premier diagramme ```mermaid d'un document (la carte mentale de
  // l'atelier, par exemple) en texte à gauche, son dessin à jour à droite ; « Enregistrer » remplace le
  // bloc dans le document (Ctrl+Z hors du champ pour annuler).
  import { onMount } from "svelte";
  import { sourceDocument, ecrireDocument, signaler } from "../../lib/edition.svelte";

  let { id, fermer }: { id: string; fermer?: () => void } = $props();
  /** Texte complet du document, dans lequel le bloc Mermaid est remplacé à l'enregistrement. */
  let source = $state("");
  /** Code Mermaid en cours d'édition ; une carte mentale minimale si le document n'a pas encore de diagramme. */
  let code = $state("");
  /** Code tel qu'il est dans le document. */
  let origine = $state("");
  /** Dernier dessin réussi : il reste affiché tant que le code saisi est invalide. */
  let svg = $state("");
  /** Erreur de lecture, d'enregistrement ou de syntaxe du diagramme (première ligne seulement). */
  let erreur = $state("");
  /** Premier bloc ```mermaid du document : c'est lui qu'on édite. */
  const BLOC = /```mermaid\n([\s\S]*?)```/;

  onMount(async () => {
    try {
      source = (await sourceDocument(id)).texte;
      code = origine = BLOC.exec(source)?.[1].trimEnd() ?? "mindmap\n  root((Sujet))\n    Idée";
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  });

  /** Compteur d'identifiants de rendu : Mermaid refuse de réutiliser un identifiant. */
  let n = 0;
  /** Délai de 300 ms avant de redessiner, pour ne pas relancer Mermaid à chaque touche. */
  let minuterie: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const c = code;
    clearTimeout(minuterie);
    minuterie = setTimeout(async () => {
      if (!c.trim()) return;
      try {
        const mermaid = (await import("mermaid")).default;
        const sombre = matchMedia("(prefers-color-scheme: dark)").matches;
        mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme: sombre ? "dark" : "default" });
        svg = (await mermaid.render(`edition-mermaid-${n++}`, c)).svg;
        erreur = "";
      } catch (e) {
        erreur = "Diagramme invalide : " + String(e instanceof Error ? e.message : e).split("\n")[0];
      }
    }, 300);
  });

  /** Remplace le bloc dans le document, ou l'ajoute en fin s'il n'y en avait pas, puis ferme l'éditeur ; Ctrl+Z annule. */
  async function enregistrer() {
    const bloc = "```mermaid\n" + code.trimEnd() + "\n```";
    const texte = BLOC.test(source) ? source.replace(BLOC, bloc) : `${source.trimEnd()}\n\n${bloc}\n`;
    try {
      await ecrireDocument(id, texte);
      signaler(`Diagramme de ${id} enregistré (Ctrl+Z pour annuler)`);
      fermer?.();
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }
</script>

<section class="editeur-mermaid">
  <div class="barre-mermaid">
    <strong>Diagramme de {id}</strong>
    <span class="discret">Mermaid : <code>mindmap</code>, <code>flowchart</code>… une ligne par nœud, l'indentation fait la hiérarchie.</span>
    <span class="espace"></span>
    {#if fermer}<button onclick={fermer}>Fermer</button>{/if}
    <button class="principal" onclick={enregistrer} disabled={code === origine || !!erreur}>Enregistrer</button>
  </div>
  <div class="volets">
    <textarea class="mono" bind:value={code} spellcheck="false" aria-label="Source Mermaid"></textarea>
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    <div class="dessin-mermaid">{@html svg}</div>
  </div>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
</section>

<style>
  .editeur-mermaid {
    border: 1px solid var(--bord-fort);
    border-radius: var(--rayon);
    margin: 12px 0;
    background: var(--fond);
  }
  .barre-mermaid {
    display: flex;
    gap: 8px;
    align-items: center;
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
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr);
    min-height: 360px;
  }
  textarea {
    border: none;
    border-right: 1px solid var(--bord);
    border-radius: 0;
    padding: 12px;
    font-size: 13px;
    line-height: 1.5;
    resize: vertical;
    min-height: 360px;
  }
  .dessin-mermaid {
    padding: 10px;
    overflow: auto;
    display: grid;
    place-items: center;
  }
  .dessin-mermaid :global(svg) {
    max-width: 100%;
    height: auto;
  }
  @media (max-width: 800px) {
    .volets {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
