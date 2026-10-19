<script lang="ts">
  // Cellule de symbole (EF-041) : en-tête (nom = lien direct, ⚓ = référence à copier), doc rendue,
  // balises, voisins, code replié.
  // @relu 2026-10-17 c9c4bd41
  import type { Cellule } from "../types/Cellule";
  import { rendre, colorer } from "../lib/markdown";
  import { reference, copier } from "../lib/hook";
  import { url } from "../lib/routeur.svelte";
  import Lien from "./Lien.svelte";
  import Balises from "./Balises.svelte";

  let { cellule, ouverte = false, langage = "rust" }: { cellule: Cellule; ouverte?: boolean; langage?: string } = $props();
  let copie = $state<"" | "copie" | "echec">("");
  const doc = $derived(cellule.doc ? rendre(cellule.doc) : "");
  // Pas de grammaire Svelte dans highlight.js : le script est du TypeScript.
  const langageCode = $derived(langage === "svelte" ? "typescript" : langage);
  const code = $derived(colorer(cellule.code, langageCode));
  const nbLignes = $derived(cellule.ligne_fin - cellule.ligne_debut + 1);
  const classeStatut = $derived(cellule.statut === "périmé" ? "perime" : cellule.statut === "sans doc" ? "sansdoc" : "ajour");

  async function hook() {
    copie = (await copier(reference(cellule.id))) ? "copie" : "echec";
    setTimeout(() => (copie = ""), 1600);
  }
  // Lien permanent vers ce symbole (REC-020) : la page du fichier, ancrée sur la cellule.
  const ancre = $derived(url.fichier(cellule.id.slice(0, cellule.id.length - cellule.nom_qualifie.length - 2), cellule.nom_qualifie));
</script>

<article class="cellule {classeStatut}" id={cellule.nom_qualifie}>
  <header>
    <span class="genre">{cellule.visibilite === "prive" ? "" : cellule.visibilite + " "}{cellule.genre}</span>
    <h3><a class="ancre-symbole" href={ancre} title="Lien direct vers ce symbole">{cellule.nom_qualifie}</a></h3>
    <span class="statut {classeStatut}">{cellule.statut}</span>
    <span class="lignes">L{cellule.ligne_debut}–{cellule.ligne_fin}</span>
    <a class="action" href={url.graphe(cellule.id)} title="Graphe local">◎</a>
    <button class="hook {copie}" onclick={hook} title="Copier la référence @wiki: à coller dans un prompt">⚓</button>
  </header>
  <pre class="signature"><code class="hljs">{@html colorer(cellule.signature, langageCode)}</code></pre>
  {#if doc}
    <div class="doc prose">{@html doc}</div>
  {:else}
    <p class="discret">Pas de bloc de doc.</p>
  {/if}
  <Balises balises={cellule.balises} />
  {#if cellule.appelants.length || cellule.appeles.length}
    <div class="voisins">
      {#if cellule.appelants.length}
        <div><span class="etiquette">appelé par</span> {#each cellule.appelants as l}<Lien lien={l} court />{/each}</div>
      {/if}
      {#if cellule.appeles.length}
        <div><span class="etiquette">appelle</span> {#each cellule.appeles as l}<Lien lien={l} court />{/each}</div>
      {/if}
    </div>
  {/if}
  <details class="code" open={ouverte}>
    <summary>Code ({nbLignes} ligne{nbLignes > 1 ? "s" : ""})</summary>
    <pre><code class="hljs">{@html code}</code></pre>
  </details>
</article>
