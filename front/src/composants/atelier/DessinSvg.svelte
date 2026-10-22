<script lang="ts">
  // Éditeur vectoriel (EF-167, REC-078) : dessiner une vignette de storyboard ou une maquette d'interface
  // — rectangles, ellipses, lignes, flèches et textes, chacun avec un libellé, déplacés, redimensionnés,
  // colorés, supprimés ; cadre de téléphone ou de navigateur au choix. Enregistré en SVG dans
  // `docs/atelier/dessins/<nom>.svg` ; le modèle des formes voyage dans l'attribut `data-wikik` pour être
  // repris tel quel. Ctrl+Z / Ctrl+Y annulent dans l'éditeur, Suppr efface la forme choisie.
  import { onMount } from "svelte";
  import { poster } from "../../lib/terminal.svelte";
  import { signaler } from "../../lib/edition.svelte";

  let { nom, titre = "", fermer }: { nom: string; titre?: string; fermer: (enregistre: boolean) => void } = $props();

  /** Outil actif de la barre ; « choisir » sélectionne, les autres créent une forme au clic ou au glisser. */
  type Outil = "choisir" | "rect" | "ellipse" | "ligne" | "fleche" | "texte";
  /**
   * Une forme du dessin, sérialisée telle quelle dans `data-wikik` : `w` et `h` peuvent être négatifs (tracé vers le haut ou la gauche),
   * `texte` est le libellé affiché au centre.
   */
  type Forme = { id: number; type: Exclude<Outil, "choisir">; x: number; y: number; w: number; h: number; texte: string; trait: string; fond: string; epaisseur: number; taille: number };
  /** Cadre dessiné autour de la maquette : aucun, téléphone ou fenêtre de navigateur. */
  type Cadre = "aucun" | "telephone" | "navigateur";

  /** Largeur du repère SVG (viewBox), indépendante de la taille affichée. */
  const LARGEUR = 800;
  /** Hauteur du repère SVG, en 16/9 avec la largeur. */
  const HAUTEUR = 450;
  /** Formes dans l'ordre de dessin : la dernière est au premier plan. */
  let formes = $state<Forme[]>([]);
  /** Cadre choisi, enregistré avec les formes. */
  let cadre = $state<Cadre>("aucun");
  /** Outil choisi dans la barre ; repasse à « choisir » après la pose d'un texte. */
  let outil = $state<Outil>("rect");
  /** Identifiant de la forme sélectionnée (poignée de taille et panneau de propriétés), ou rien. */
  let choisie = $state<number | null>(null);
  /** Couleur de trait appliquée aux formes créées ensuite. */
  let trait = $state("#1d2026");
  /** Couleur de fond des formes créées ensuite, si `sansFond` est décoché. */
  let fond = $state("#ffffff");
  /** Formes créées sans remplissage (fond `none`), le cas le plus courant pour une maquette. */
  let sansFond = $state(true);
  /** Épaisseur de trait des formes créées ensuite. */
  let epaisseur = $state(2);
  /** Erreur de lecture ou d'enregistrement du dessin. */
  let erreur = $state("");
  /** Élément SVG dessiné : sert au calcul des coordonnées et au clone enregistré. */
  let svg: SVGSVGElement;
  /** Prochain identifiant de forme ; repris après le plus grand identifiant d'un dessin rouvert. */
  let suivant = 1;

  // Annulation locale : instantanés du modèle.
  /** Instantanés antérieurs du modèle, 100 au plus. */
  let passe: string[] = [];
  /** Instantanés annulés, vidés dès qu'une nouvelle modification est mémorisée. */
  let futur: string[] = [];
  /** Instantané sérialisé des formes et du cadre (copie détachée du proxy Svelte). */
  const instantane = () => JSON.stringify({ formes: $state.snapshot(formes), cadre });
  /** Mémorise l'état avant une modification ; à appeler juste avant de changer le modèle. */
  function memoriser() {
    passe.push(instantane());
    if (passe.length > 100) passe.shift();
    futur = [];
  }
  /** Remet un instantané en place et désélectionne (la forme choisie peut ne plus exister). */
  function restaurer(t: string) {
    const v = JSON.parse(t);
    formes = v.formes;
    cadre = v.cadre;
    choisie = null;
  }
  /** Ctrl+Z dans l'éditeur : revient à l'instantané précédent. */
  function annuler() {
    const t = passe.pop();
    if (!t) return;
    futur.push(instantane());
    restaurer(t);
  }
  /** Ctrl+Y ou Ctrl+Maj+Z dans l'éditeur : refait ce qui vient d'être annulé. */
  function retablir() {
    const t = futur.pop();
    if (!t) return;
    passe.push(instantane());
    restaurer(t);
  }

  onMount(async () => {
    try {
      const r = await fetch(`/api/dessin/lire?nom=${encodeURIComponent(nom)}`);
      const texte: string | null = r.ok ? await r.json() : null;
      if (!texte) return;
      const doc = new DOMParser().parseFromString(texte, "image/svg+xml");
      const donnees = doc.documentElement.getAttribute("data-wikik");
      if (!donnees) return;
      const v = JSON.parse(donnees);
      formes = v.formes ?? [];
      cadre = v.cadre ?? "aucun";
      suivant = Math.max(0, ...formes.map((f) => f.id)) + 1;
    } catch (e) {
      erreur = "Dessin illisible : " + String(e);
    }
  });

  /** Position du pointeur dans le repère du SVG, arrondie au pixel, quel que soit le zoom de l'affichage. */
  function point(e: PointerEvent) {
    const m = svg.getScreenCTM()!.inverse();
    const p = new DOMPoint(e.clientX, e.clientY).matrixTransform(m);
    return { x: Math.round(p.x), y: Math.round(p.y) };
  }

  /** Geste en cours entre l'appui et le relâchement : création, déplacement (décalage du point saisi) ou redimensionnement. */
  let geste: { mode: "creer" | "deplacer" | "taille"; id: number; dx: number; dy: number } | null = null;

  /**
   * Appui du pointeur : poignée → redimensionner ; outil « choisir » → sélectionner et déplacer ; autre outil → créer une forme.
   * Un texte se pose d'un clic, sans glisser, et rend la main à l'outil « choisir ». Le pointeur est capturé pour suivre le geste hors du dessin.
   */
  function bas(e: PointerEvent) {
    svg.focus();
    const p = point(e);
    const cible = (e.target as Element).closest("[data-id]");
    const poignee = (e.target as Element).closest("[data-poignee]");
    if (poignee && choisie !== null) {
      memoriser();
      geste = { mode: "taille", id: choisie, dx: 0, dy: 0 };
    } else if (outil === "choisir") {
      if (cible) {
        const id = Number(cible.getAttribute("data-id"));
        choisie = id;
        const f = formes.find((x) => x.id === id)!;
        memoriser();
        geste = { mode: "deplacer", id, dx: p.x - f.x, dy: p.y - f.y };
      } else choisie = null;
    } else {
      memoriser();
      const f: Forme = {
        id: suivant++,
        type: outil,
        x: p.x,
        y: p.y,
        w: outil === "texte" ? 0 : 0,
        h: 0,
        texte: outil === "texte" ? "Texte" : "",
        trait,
        fond: sansFond ? "none" : fond,
        epaisseur,
        taille: 16,
      };
      formes.push(f);
      choisie = f.id;
      geste = outil === "texte" ? null : { mode: "creer", id: f.id, dx: 0, dy: 0 };
      if (outil === "texte") outil = "choisir";
    }
    if (geste) svg.setPointerCapture(e.pointerId);
  }

  /** Suit le geste : déplace la forme ou change sa taille en tirant son coin opposé à l'origine. */
  function bouge(e: PointerEvent) {
    if (!geste) return;
    const p = point(e);
    const f = formes.find((x) => x.id === geste!.id);
    if (!f) return;
    if (geste.mode === "deplacer") {
      f.x = p.x - geste.dx;
      f.y = p.y - geste.dy;
    } else {
      f.w = p.x - f.x;
      f.h = p.y - f.y;
    }
  }

  /** Fin du geste ; une forme créée d'un simple clic reçoit une taille par défaut au lieu de rester invisible. */
  function haut() {
    if (geste?.mode === "creer") {
      const f = formes.find((x) => x.id === geste!.id);
      // Un simple clic ne crée pas de forme invisible : elle prend une taille par défaut.
      if (f && Math.abs(f.w) < 4 && Math.abs(f.h) < 4) {
        f.w = f.type === "ligne" || f.type === "fleche" ? 120 : 140;
        f.h = f.type === "ligne" || f.type === "fleche" ? 0 : 70;
      }
    }
    geste = null;
  }

  /** Forme sélectionnée, éditée par le panneau de propriétés. */
  const selection = $derived(formes.find((f) => f.id === choisie) ?? null);
  /** Applique un changement de propriété à la forme sélectionnée, annulable. */
  function appliquer(change: Partial<Forme>) {
    if (!selection) return;
    memoriser();
    Object.assign(selection, change);
  }
  /** Supprime la forme sélectionnée (bouton ou touche Suppr). */
  function supprimer() {
    if (choisie === null) return;
    memoriser();
    formes = formes.filter((f) => f.id !== choisie);
    choisie = null;
  }
  /** Passe la forme sélectionnée au premier plan, en la plaçant en fin de liste. */
  function premierPlan() {
    if (!selection) return;
    memoriser();
    formes = [...formes.filter((f) => f.id !== selection.id), selection];
  }

  /**
   * Raccourcis clavier de l'éditeur, ignorés quand on tape dans un champ : Ctrl+Z / Ctrl+Y, Suppr.
   * La propagation est arrêtée pour que le Ctrl+Z global (journal d'annulation, ADR-0016) ne réagisse pas en même temps.
   */
  function touche(e: KeyboardEvent) {
    const champ = (e.target as HTMLElement).closest("input, textarea, select");
    if ((e.ctrlKey || e.metaKey) && !champ) {
      const k = e.key.toLowerCase();
      if (k === "z" || k === "y") {
        e.preventDefault();
        e.stopPropagation();
        if (k === "y" || e.shiftKey) retablir();
        else annuler();
      }
      return;
    }
    if ((e.key === "Delete" || e.key === "Backspace") && !champ) {
      e.preventDefault();
      e.stopPropagation();
      supprimer();
    }
  }

  /** Boîte englobante normalisée d'une forme (largeur et hauteur positives), pour les rectangles, ellipses et poignées. */
  const boite = (f: Forme) => ({ x: Math.min(f.x, f.x + f.w), y: Math.min(f.y, f.y + f.h), w: Math.abs(f.w), h: Math.abs(f.h) });

  /**
   * Enregistre le dessin en SVG autonome : la sélection est retirée d'abord pour ne pas sortir dans le fichier,
   * le modèle des formes est joint dans `data-wikik` pour pouvoir rouvrir le dessin ; ferme l'éditeur si l'écriture réussit.
   */
  async function enregistrer() {
    erreur = "";
    choisie = null;
    await Promise.resolve();
    const clone = svg.cloneNode(true) as SVGSVGElement;
    clone.querySelectorAll("[data-poignee], .selection-dessin").forEach((n) => n.remove());
    clone.removeAttribute("tabindex");
    clone.removeAttribute("class");
    clone.setAttribute("xmlns", "http://www.w3.org/2000/svg");
    clone.setAttribute("data-wikik", JSON.stringify({ formes: $state.snapshot(formes), cadre }));
    const texte = new XMLSerializer().serializeToString(clone).replace(/<!--.*?-->/gs, "");
    try {
      await poster("/api/dessin", { nom, svg: texte });
      signaler(`Dessin ${nom} enregistré (Ctrl+Z hors de l'éditeur pour annuler)`);
      fermer(true);
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }

  /** Boutons de la barre d'outils : outil, symbole affiché, bulle d'aide. */
  const OUTILS: [Outil, string, string][] = [
    ["choisir", "↖", "Choisir, déplacer, redimensionner"],
    ["rect", "▭", "Rectangle (bouton, zone, écran)"],
    ["ellipse", "◯", "Ellipse"],
    ["ligne", "╱", "Ligne"],
    ["fleche", "→", "Flèche"],
    ["texte", "T", "Texte"],
  ];
</script>

<div class="fond-dessin" role="presentation">
  <div class="dessin" role="dialog" aria-label="Éditeur de dessin {nom}" tabindex="-1" onkeydown={touche}>
    <header class="barre-dessin">
      <strong>{titre || nom}</strong>
      <span class="outils-dessin" role="radiogroup" aria-label="Outil">
        {#each OUTILS as [o, glyphe, aide] (o)}
          <button class:actif={outil === o} role="radio" aria-checked={outil === o} title={aide} onclick={() => (outil = o)}>{glyphe}</button>
        {/each}
      </span>
      <label title="Couleur du trait">Trait <input type="color" bind:value={trait} onchange={() => appliquer({ trait })} /></label>
      <label title="Remplissage"><input type="checkbox" bind:checked={sansFond} onchange={() => appliquer({ fond: sansFond ? "none" : fond })} /> vide</label>
      <input type="color" bind:value={fond} disabled={sansFond} title="Couleur de remplissage" onchange={() => appliquer({ fond: sansFond ? "none" : fond })} />
      <label title="Épaisseur">Épaisseur <select bind:value={epaisseur} onchange={() => appliquer({ epaisseur })}>{#each [1, 2, 3, 5, 8] as n}<option value={n}>{n}</option>{/each}</select></label>
      <label title="Cadre de fond">Cadre
        <select bind:value={cadre} onchange={memoriser}>
          <option value="aucun">aucun</option>
          <option value="telephone">téléphone</option>
          <option value="navigateur">navigateur</option>
        </select>
      </label>
    </header>
    {#if selection}
      <div class="proprietes">
        <label>Texte <input bind:value={selection.texte} placeholder="libellé de la forme" /></label>
        <label>Taille <input type="number" min="8" max="72" bind:value={selection.taille} /></label>
        <button onclick={premierPlan} title="Mettre au premier plan">Premier plan</button>
        <button class="danger-texte" onclick={supprimer} title="Suppr">Supprimer</button>
      </div>
    {/if}
    <svg
      bind:this={svg}
      class="toile"
      viewBox="0 0 {LARGEUR} {HAUTEUR}"
      tabindex="0"
      role="application"
      aria-label="Zone de dessin"
      onpointerdown={bas}
      onpointermove={bouge}
      onpointerup={haut}
    >
      <defs>
        <marker id="pointe-{nom}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M0 0L10 5L0 10z" fill="context-stroke" />
        </marker>
      </defs>
      <rect x="0" y="0" width={LARGEUR} height={HAUTEUR} fill="#ffffff" />
      {#if cadre === "telephone"}
        <g fill="none" stroke="#8a919c" stroke-width="3">
          <rect x="300" y="12" width="200" height="426" rx="26" />
          <line x1="370" y1="30" x2="430" y2="30" stroke-linecap="round" />
        </g>
      {:else if cadre === "navigateur"}
        <g fill="none" stroke="#8a919c" stroke-width="2">
          <rect x="20" y="12" width="760" height="426" rx="8" />
          <line x1="20" y1="44" x2="780" y2="44" />
          <circle cx="40" cy="28" r="5" /><circle cx="58" cy="28" r="5" /><circle cx="76" cy="28" r="5" />
          <rect x="100" y="19" width="560" height="18" rx="9" />
        </g>
      {/if}
      {#each formes as f (f.id)}
        {@const b = boite(f)}
        <g data-id={f.id} class="forme" style="cursor: {outil === 'choisir' ? 'move' : 'crosshair'}">
          {#if f.type === "rect"}
            <rect x={b.x} y={b.y} width={b.w} height={b.h} rx="6" fill={f.fond} stroke={f.trait} stroke-width={f.epaisseur} />
          {:else if f.type === "ellipse"}
            <ellipse cx={b.x + b.w / 2} cy={b.y + b.h / 2} rx={b.w / 2} ry={b.h / 2} fill={f.fond} stroke={f.trait} stroke-width={f.epaisseur} />
          {:else if f.type === "ligne" || f.type === "fleche"}
            <line x1={f.x} y1={f.y} x2={f.x + f.w} y2={f.y + f.h} stroke={f.trait} stroke-width={f.epaisseur} stroke-linecap="round" marker-end={f.type === "fleche" ? `url(#pointe-${nom})` : undefined} />
            <line x1={f.x} y1={f.y} x2={f.x + f.w} y2={f.y + f.h} stroke="transparent" stroke-width="14" />
          {/if}
          {#if f.texte}
            <text x={f.type === "texte" ? f.x : b.x + b.w / 2} y={f.type === "texte" ? f.y : b.y + b.h / 2} fill={f.trait} font-size={f.taille} font-family="system-ui, sans-serif" text-anchor={f.type === "texte" ? "start" : "middle"} dominant-baseline="middle">{f.texte}</text>
          {/if}
        </g>
      {/each}
      {#if selection}
        {@const b = selection.type === "texte" ? { x: selection.x - 4, y: selection.y - selection.taille, w: Math.max(40, selection.texte.length * selection.taille * 0.6) + 8, h: selection.taille * 2 } : boite(selection)}
        <rect class="selection-dessin" x={b.x - 4} y={b.y - 4} width={b.w + 8} height={b.h + 8} fill="none" stroke="#2563eb" stroke-dasharray="5 4" pointer-events="none" />
        {#if selection.type !== "texte"}
          <rect data-poignee="1" x={selection.x + selection.w - 6} y={selection.y + selection.h - 6} width="12" height="12" fill="#2563eb" style="cursor: nwse-resize" />
        {/if}
      {/if}
    </svg>
    <footer class="actions">
      <button class="principal" onclick={enregistrer}>Enregistrer</button>
      <button onclick={() => fermer(false)}>Fermer</button>
      <button class="discret-bouton" onclick={() => { memoriser(); formes = []; choisie = null; }}>Tout effacer</button>
      <span class="discret">Glisser pour tracer · ↖ pour déplacer (poignée bleue : taille) · Suppr efface · Ctrl+Z annule</span>
      {#if erreur}<span class="erreur">{erreur}</span>{/if}
    </footer>
  </div>
</div>

<style>
  .fond-dessin {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 45%);
    padding: 16px;
  }
  .dessin {
    width: min(1100px, 100%);
    max-height: 100%;
    overflow: auto;
    background: var(--fond);
    border-radius: var(--rayon);
    border: 1px solid var(--bord-fort);
    padding: 10px 14px;
  }
  .barre-dessin,
  .proprietes {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 8px;
    font-size: 13px;
  }
  .outils-dessin {
    display: inline-flex;
    border: 1px solid var(--bord-fort);
    border-radius: var(--rayon);
    overflow: hidden;
  }
  .outils-dessin button {
    width: 34px;
    height: 30px;
    border: none;
    border-right: 1px solid var(--bord);
    background: var(--fond-2);
    color: var(--texte);
    font-size: 16px;
    cursor: pointer;
  }
  .outils-dessin button.actif {
    background: var(--accent);
    color: white;
  }
  .toile {
    width: 100%;
    height: auto;
    border: 1px solid var(--bord);
    border-radius: 6px;
    touch-action: none;
    outline: none;
  }
  .toile:focus-visible {
    box-shadow: 0 0 0 2px var(--accent);
  }
  .danger-texte {
    color: var(--rouge);
  }
</style>
