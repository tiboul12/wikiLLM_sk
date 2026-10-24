<script lang="ts">
  // Champ à recherche dynamique (EF-172, REC-113) : les éléments qui correspondent s'affichent sous le champ au fil
  // de la frappe, triés par la recherche floue ; ↑ ↓ pour choisir, Entrée ou clic pour valider, Échap pour fermer.
  // `choisir` reçoit l'élément retenu, ou le texte saisi tel quel quand rien n'est sélectionné (Entrée sans avoir
  // choisi au clavier) ; un texte égal à un identifiant, à la casse près, vaut cet élément.
  import { chercher, normaliser, type Cherchable } from "../lib/floue";

  let {
    elements,
    choisir,
    valeur = $bindable(""),
    placeholder = "",
    libelle = "Rechercher",
  }: { elements: Cherchable[]; choisir: (e: Cherchable | string) => void; valeur?: string; placeholder?: string; libelle?: string } = $props();

  /** Liste ouverte sous le champ. */
  let ouverte = $state(false);
  /** Rang de l'élément surligné au clavier ; -1 : aucun, Entrée valide le texte saisi. */
  let rang = $state(-1);
  /** Éléments qui correspondent au texte saisi. */
  const resultats = $derived(valeur.trim() ? chercher(valeur, elements) : []);
  $effect(() => {
    resultats;
    rang = -1;
  });

  /** Valide l'élément `e`, celui surligné, celui dont l'identifiant est le texte saisi, ou à défaut le texte. */
  function valider(e?: Cherchable) {
    ouverte = false;
    const exact = resultats.find((x) => normaliser(x.id) === normaliser(valeur));
    choisir(e ?? resultats[rang] ?? exact ?? valeur.trim());
  }
  /** Navigation au clavier dans la liste.
   * @relu 2026-10-24 6f1cee04
   */
  function clavier(ev: KeyboardEvent) {
    if (ev.key === "ArrowDown") rang = Math.min(rang + 1, resultats.length - 1);
    else if (ev.key === "ArrowUp") rang = Math.max(rang - 1, -1);
    else if (ev.key === "Enter") valider();
    else if (ev.key === "Escape") ouverte = false;
    else return;
    ev.preventDefault();
    ouverte = ev.key !== "Escape" && ev.key !== "Enter";
  }
</script>

<span class="champ-floue">
  <input type="search" bind:value={valeur} {placeholder} aria-label={libelle} autocomplete="off" role="combobox" aria-expanded={ouverte && resultats.length > 0} aria-controls="liste-floue"
    oninput={() => (ouverte = true)} onfocus={() => (ouverte = true)} onblur={() => setTimeout(() => (ouverte = false), 150)} onkeydown={clavier} />
  {#if ouverte && resultats.length}
    <ul class="liste-floue" id="liste-floue" role="listbox">
      {#each resultats as e, i (e.id)}
        <li role="option" aria-selected={i === rang} class:surligne={i === rang}>
          <button type="button" onmousedown={(ev) => ev.preventDefault()} onclick={() => valider(e)}>
            <span class="id">{e.id}</span> <span class="lib">{e.libelle === e.id ? "" : e.libelle}</span>{#if e.detail}<small>{e.detail}</small>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</span>

<style>
  .champ-floue {
    position: relative;
    display: inline-block;
    flex: 1 1 280px;
    min-width: 0;
  }
  .champ-floue input {
    width: 100%;
  }
  .liste-floue {
    position: absolute;
    z-index: 20;
    left: 0;
    right: 0;
    top: calc(100% + 2px);
    max-height: 320px;
    overflow-y: auto;
    margin: 0;
    padding: 4px;
    list-style: none;
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    background: var(--fond);
    box-shadow: 0 6px 18px rgb(0 0 0 / 0.18);
  }
  .liste-floue button {
    display: flex;
    gap: 6px;
    align-items: baseline;
    width: 100%;
    padding: 5px 8px;
    border: 0;
    border-radius: 5px;
    background: none;
    color: var(--texte);
    font: inherit;
    font-size: 13.5px;
    text-align: left;
    cursor: pointer;
  }
  .surligne button,
  .liste-floue button:hover {
    background: var(--accent-fond);
  }
  .id {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--texte-2);
    flex: none;
  }
  .lib {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  small {
    margin-left: auto;
    color: var(--texte-2);
    flex: none;
  }
</style>
