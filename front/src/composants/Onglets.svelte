<script lang="ts">
  // Barre d'onglets au-dessus du contenu (EF-151) : un clic affiche l'onglet, × ou clic du milieu le ferme,
  // l'icône ↗ le détache dans une fenêtre ; ☆ met la page en favori (EF-157) ; on les réordonne en les
  // glissant. La barre défile à la molette, sans barre de défilement visible qui recouvrirait les titres (REC-088).
  // @relu 2026-10-24 c8888d71
  import { onglets, activer, fermer, deplacer, detacher } from "../lib/onglets.svelte";
  import { estFavori, basculerFavori } from "../lib/edition.svelte";
  import { statique } from "../lib/statique";

  /** Barre d'onglets, pour faire défiler l'onglet actif dans la vue. */
  let barre = $state<HTMLElement | null>(null);
  // L'onglet actif reste visible quand la barre défile.
  $effect(() => {
    const i = onglets.actif;
    barre?.querySelectorAll(".onglet")[i]?.scrollIntoView({ block: "nearest", inline: "nearest" });
  });
  /**
   * La barre n'a pas de barre de défilement visible, qui recouvrait les titres (REC-088) : la molette la fait défiler
   * de côté, et l'onglet actif est toujours ramené dans la vue.
   */
  function defiler(e: WheelEvent) {
    if (!barre || Math.abs(e.deltaY) <= Math.abs(e.deltaX)) return;
    barre.scrollLeft += e.deltaY;
    e.preventDefault();
  }
  // Écouteur non passif, pour que la molette ne fasse pas aussi défiler la page.
  $effect(() => {
    const b = barre;
    b?.addEventListener("wheel", defiler, { passive: false });
    return () => b?.removeEventListener("wheel", defiler);
  });
  /** Rang de l'onglet en cours de glisser. */
  let glisse = $state<number | null>(null);
  /** Rang de l'onglet survolé pendant le glisser : l'onglet glissé sera posé à sa place. */
  let cible = $state<number | null>(null);
</script>

{#if onglets.liste.length}
  <div class="onglets" role="tablist" aria-label="Pages ouvertes" bind:this={barre}>
    {#each onglets.liste as o, i (i + o.url)}
      <div
        class="onglet"
        class:actif={i === onglets.actif}
        class:cible={cible === i && glisse !== i}
        role="tab"
        tabindex="0"
        aria-selected={i === onglets.actif}
        title={o.titre}
        draggable="true"
        ondragstart={(e) => {
          glisse = i;
          e.dataTransfer?.setData("text/plain", String(i));
        }}
        ondragover={(e) => {
          e.preventDefault();
          cible = i;
        }}
        ondragleave={() => (cible = null)}
        ondrop={(e) => {
          e.preventDefault();
          if (glisse !== null) deplacer(glisse, i);
          glisse = cible = null;
        }}
        ondragend={() => (glisse = cible = null)}
        onclick={() => activer(i)}
        onauxclick={(e) => e.button === 1 && fermer(i)}
        onkeydown={(e) => e.key === "Enter" && activer(i)}
      >
        <span class="titre-onglet">{o.titre}</span>
        {#if !statique}
          {@const fav = estFavori(o.url)}
          <button class="action-onglet" class:favori={fav} title={fav ? "Retirer des favoris" : "Mettre en favori"} aria-label={fav ? "Retirer des favoris" : "Mettre en favori"} aria-pressed={fav} onclick={(e) => { e.stopPropagation(); basculerFavori(o.url, o.titre); }}
            ><svg width="12" height="12" viewBox="0 0 24 24" fill={fav ? "currentColor" : "none"} stroke="currentColor" stroke-width="2" stroke-linejoin="round" aria-hidden="true"><path d="M12 3l2.8 5.7 6.2.9-4.5 4.4 1.1 6.2L12 17.3 6.4 20.2l1.1-6.2L3 9.6l6.2-.9z" /></svg></button
          >
        {/if}
        <button class="action-onglet" title="Détacher dans une fenêtre" aria-label="Détacher" onclick={(e) => { e.stopPropagation(); detacher(i); }}
          ><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5" /></svg></button
        >
        <button class="action-onglet" title="Fermer l'onglet" aria-label="Fermer" onclick={(e) => { e.stopPropagation(); fermer(i); }}>×</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .onglets {
    display: flex;
    max-width: 100%;
    min-width: 0;
    gap: 2px;
    overflow-x: auto;
    border-bottom: 1px solid var(--bord);
    background: var(--fond-2);
    position: sticky;
    top: 54px;
    z-index: 8;
    scrollbar-width: none;
  }
  .onglets::-webkit-scrollbar {
    display: none;
  }
  .onglet {
    display: flex;
    align-items: center;
    gap: 4px;
    max-width: 240px;
    min-width: 0;
    min-height: 36px;
    padding: 8px 6px 8px 12px;
    font-size: 13px;
    color: var(--texte-2);
    border-right: 1px solid var(--bord);
    cursor: pointer;
    user-select: none;
    flex-shrink: 0;
  }
  .onglet:hover {
    background: var(--fond-3);
  }
  .onglet.actif {
    background: var(--fond);
    color: var(--texte);
    box-shadow: inset 0 2px 0 var(--accent);
  }
  .onglet.cible {
    box-shadow: inset 2px 0 0 var(--accent);
  }
  .titre-onglet {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .action-onglet {
    border: none;
    background: none;
    color: var(--texte-2);
    cursor: pointer;
    padding: 0 4px;
    font-size: 14px;
    line-height: 1;
    border-radius: 4px;
    opacity: 0;
  }
  .onglet:hover .action-onglet,
  .onglet.actif .action-onglet {
    opacity: 1;
  }
  .action-onglet.favori {
    color: var(--orange);
    opacity: 1;
  }
  .action-onglet:hover {
    background: var(--fond-3);
    color: var(--texte);
  }
  @media (hover: none) {
    .action-onglet {
      opacity: 1;
    }
  }
</style>
