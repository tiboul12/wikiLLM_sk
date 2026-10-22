<script lang="ts">
  // Historique git (EF-166, REC-067), à la GitKraken : le graphe des branches (un couloir de couleur par
  // branche, fusions comprises), les branches et tags sur leurs commits, la situation de la branche
  // (modifications non commitées, commits à pousser ou à récupérer) ; un commit ouvre son message, ses
  // fichiers et le diff de chacun. `?commit=` ou `?ref=` sélectionne un commit.
  import { tick } from "svelte";
  import type { Historique } from "../types/Historique";
  import type { DetailCommit } from "../types/DetailCommit";
  import { couloirs, TEINTES } from "../lib/git";
  import { routeur } from "../lib/routeur.svelte";
  import { autoLien } from "../lib/connus.svelte";

  /** Commits, références et situation de la branche ; vide hors d'un dépôt git (l'erreur le dit). */
  let historique = $state<Historique | null>(null);
  /** Erreur de lecture de l'historique ou d'un commit. */
  let erreur = $state("");
  /** Nombre de commits demandés : au-delà, le graphe devient lent à tracer. */
  let limite = $state(400);
  /** Filtre sur le sujet, le début du hash, l'auteur ou une référence. */
  let filtre = $state("");
  /** Hash du commit sélectionné. */
  let choisi = $state<string | null>(null);
  /** Message et fichiers du commit sélectionné. */
  let detail = $state<DetailCommit | null>(null);
  /** Fichier du commit dont le diff est affiché. */
  let fichier = $state<string | null>(null);
  /** Diff du fichier choisi, ou le message d'erreur à sa place. */
  let diff = $state<string | null>(null);

  /** GET JSON qui lève l'erreur `{erreur}` du serveur, ou le code HTTP à défaut. */
  async function lire<T>(chemin: string): Promise<T> {
    const r = await fetch(chemin);
    const v = await r.json().catch(() => ({}));
    if (!r.ok) throw new Error(v.erreur ?? `${r.status}`);
    return v as T;
  }
  /**
   * Charge l'historique puis sélectionne le commit désigné par `?commit=` ou celui que porte `?ref=` (branche ou tag),
   * en le faisant défiler au centre.
   */
  async function charger() {
    try {
      historique = await lire<Historique>(`/api/git/historique?limite=${limite}`);
      const p = new URLSearchParams(location.search);
      const ref = p.get("ref");
      const c = p.get("commit") ?? (ref ? historique.commits.find((x) => x.refs.some((r) => r === ref || r.endsWith(" " + ref) || r === `HEAD → ${ref}`))?.hash : null);
      if (c) await choisir(historique.commits.find((x) => x.hash.startsWith(c))?.hash ?? c, true);
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }
  $effect(() => {
    routeur.route;
    charger();
  });

  /** Lignes du graphe : couloir de chaque commit et traits qui la traversent. */
  const lignes = $derived(historique ? couloirs(historique.commits) : []);
  /** Couloirs affichés : 14 au plus pour que le message reste lisible ; au-delà, le graphe est rogné. */
  const largeur = $derived(Math.min(Math.max(1, ...lignes.map((l) => l.largeur)), 14));
  /** Lignes qui passent le filtre ; le graphe garde ses couloirs, quitte à montrer des traits interrompus. */
  const visibles = $derived(
    filtre.trim()
      ? lignes.filter((l) => {
          const q = filtre.toLowerCase();
          return l.commit.sujet.toLowerCase().includes(q) || l.commit.hash.startsWith(q) || l.commit.auteur.toLowerCase().includes(q) || l.commit.refs.some((r) => r.toLowerCase().includes(q));
        })
      : lignes,
  );

  /** Sélectionne un commit, charge son détail et, depuis une adresse, le fait venir au centre. */
  async function choisir(hash: string, defiler = false) {
    choisi = hash;
    detail = null;
    fichier = diff = null;
    try {
      detail = await lire<DetailCommit>(`/api/git/commit?hash=${hash}`);
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
    if (defiler) {
      await tick();
      document.getElementById("c-" + hash)?.scrollIntoView({ block: "center" });
    }
  }
  /** Affiche le diff d'un fichier du commit sélectionné. */
  async function voirDiff(chemin: string) {
    if (!detail) return;
    fichier = chemin;
    diff = null;
    try {
      diff = await lire<string>(`/api/git/diff?hash=${detail.hash}&chemin=${encodeURIComponent(chemin)}`);
    } catch (e) {
      diff = String(e instanceof Error ? e.message : e);
    }
  }

  /** Hauteur d'une ligne du graphe, en pixels (doit suivre la hauteur des lignes du tableau). */
  const H = 28;
  /** Largeur d'un couloir, en pixels. */
  const W = 16;
  /** Abscisse du centre d'un couloir. */
  const x = (k: number) => k * W + W / 2;
  /** Ordonnée d'une position dans la ligne : 0 haut, 1 milieu (le commit), 2 bas. */
  const y = (p: number) => (p * H) / 2;
  /** Tracé SVG d'un trait : droit dans un même couloir, en courbe d'un couloir à l'autre. */
  const chemin = (t: { de: number; a: number; depuis: number; vers: number }) => {
    const [x1, y1, x2, y2] = [x(t.de), y(t.depuis), x(t.a), y(t.vers)];
    if (x1 === x2) return `M${x1} ${y1}L${x2} ${y2}`;
    const m = (y1 + y2) / 2;
    return `M${x1} ${y1}C${x1} ${m} ${x2} ${m} ${x2} ${y2}`;
  };
  /** Genre d'une référence pour sa couleur : HEAD, branche distante (`origin/…`), tag ou branche locale. */
  const genreRef = (r: string) => (r.startsWith("HEAD") ? "head" : r.includes("/") ? "distante" : /^v\d|^v[0-9.]/.test(r) || historique?.tags.includes(r) ? "tag" : "branche");
  /** Classe d'une ligne de diff : ajout, retrait, en-tête de bloc ou de fichier. */
  const classeLigne = (l: string) => (l.startsWith("+++") || l.startsWith("---") ? "fichier" : l.startsWith("+") ? "ajout" : l.startsWith("-") ? "retrait" : l.startsWith("@@") ? "bloc" : l.startsWith("diff ") || l.startsWith("index ") ? "fichier" : "");
</script>

<h1>Historique git</h1>
{#if erreur}<p class="erreur">{erreur}</p>{/if}
{#if !historique}
  <p class="discret">Chargement…</p>
{:else}
  <div class="situation">
    <span class="pastille">branche <strong>{historique.branche || "HEAD détaché"}</strong></span>
    {#if historique.modifies}<span class="pastille attente">{historique.modifies} fichier(s) modifié(s), non commité(s)</span>{/if}
    {#if historique.en_avance}<span class="pastille attente">{historique.en_avance} commit(s) à pousser</span>{/if}
    {#if historique.en_retard}<span class="pastille attente">{historique.en_retard} commit(s) à récupérer</span>{/if}
    {#if !historique.modifies && !historique.en_avance && !historique.en_retard}<span class="pastille ok">à jour</span>{/if}
    <span class="discret">{historique.commits.length} commits · {historique.branches.length} branches · {historique.tags.length} tags</span>
    <input type="search" bind:value={filtre} placeholder="Filtrer : message, auteur, hash, branche…" aria-label="Filtrer l'historique" />
  </div>

  <div class="git" class:avec-detail={!!choisi}>
    <div class="journal" role="list">
      {#each visibles as l (l.commit.hash)}
        <button class="commit" class:choisi={choisi === l.commit.hash} id="c-{l.commit.hash}" role="listitem" onclick={() => choisir(l.commit.hash)}>
          <svg class="graphe-git" width={largeur * W} height={H} aria-hidden="true">
            {#if !filtre.trim()}
              {#each l.traits as t}
                <path d={chemin(t)} stroke={TEINTES[t.couleur % TEINTES.length]} stroke-width="2" fill="none" />
              {/each}
            {/if}
            <circle cx={x(l.couloir)} cy={H / 2} r={l.commit.parents.length > 1 ? 4.5 : 5} fill={l.commit.parents.length > 1 ? "var(--fond)" : TEINTES[(l.traits.find((t) => t.de === l.couloir && t.depuis === 1)?.couleur ?? l.couloir) % TEINTES.length]} stroke={TEINTES[(l.traits.find((t) => t.de === l.couloir && t.depuis === 1)?.couleur ?? l.couloir) % TEINTES.length]} stroke-width="2.5" />
          </svg>
          <span class="refs">
            {#each l.commit.refs as r}<span class="ref ref-{genreRef(r)}">{r}</span>{/each}
          </span>
          <span class="sujet" title={l.commit.sujet}>{l.commit.sujet}</span>
          <span class="auteur">{l.commit.auteur}</span>
          <span class="date">{l.commit.date}</span>
          <code class="hash">{l.commit.hash.slice(0, 7)}</code>
        </button>
      {/each}
      {#if historique.tronque}
        <button class="plus" onclick={() => { limite += 400; charger(); }}>Charger {400} commits de plus</button>
      {/if}
    </div>

    {#if choisi}
      <aside class="detail-commit">
        <button class="fermer" onclick={() => (choisi = null)} aria-label="Fermer le détail">×</button>
        {#if !detail}
          <p class="discret">Chargement…</p>
        {:else}
          <p class="discret"><code>{detail.hash.slice(0, 12)}</code> · {detail.auteur} · {detail.date}</p>
          <pre class="message">{detail.message}</pre>
          <h3>{detail.fichiers.length} fichier(s)</h3>
          <ul class="fichiers-commit">
            {#each detail.fichiers as f (f.chemin)}
              <li>
                <button class:actif={fichier === f.chemin} onclick={() => voirDiff(f.chemin)}>
                  <span class="statut-fichier s-{f.statut}">{f.statut}</span>
                  <span class="chemin">{f.chemin}</span>
                  <span class="ajouts">+{f.ajouts}</span><span class="retraits">−{f.suppressions}</span>
                </button>
                {#if autoLien(f.chemin)}<a class="wiki" href={autoLien(f.chemin)} title="Ouvrir dans le wiki">↗</a>{/if}
              </li>
            {/each}
          </ul>
          {#if fichier}
            <h3>Diff de {fichier.split("/").pop()}</h3>
            {#if diff === null}<p class="discret">Chargement…</p>{:else}
              <pre class="diff">{#each diff.split("\n") as ligne}<span class={classeLigne(ligne)}>{ligne}{"\n"}</span>{/each}</pre>
            {/if}
          {/if}
        {/if}
      </aside>
    {/if}
  </div>
{/if}

<style>
  :global(.contenu:has(.git)) {
    max-width: none;
  }
  .situation {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }
  .situation input {
    margin-left: auto;
    min-width: 16rem;
  }
  .pastille.attente {
    background: var(--orange-fond);
    color: var(--orange);
  }
  .pastille.ok {
    background: var(--vert-fond);
    color: var(--vert);
  }
  .git {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
    align-items: start;
  }
  .git.avec-detail {
    grid-template-columns: minmax(0, 1.4fr) minmax(320px, 1fr);
  }
  .journal {
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    overflow: hidden;
  }
  .commit {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 28px;
    padding: 0 10px 0 4px;
    border: none;
    border-bottom: 1px solid var(--fond-3);
    background: var(--fond);
    color: var(--texte);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }
  .commit:hover {
    background: var(--fond-2);
  }
  .commit.choisi {
    background: var(--accent-fond);
  }
  .graphe-git {
    flex-shrink: 0;
  }
  .refs {
    display: inline-flex;
    gap: 4px;
    flex-shrink: 0;
  }
  .ref {
    padding: 0 6px;
    border-radius: 4px;
    font-size: 11.5px;
    font-weight: 600;
    white-space: nowrap;
    border: 1px solid;
  }
  .ref-head,
  .ref-branche {
    color: var(--accent);
    border-color: var(--accent);
  }
  .ref-head {
    background: var(--accent);
    color: white;
  }
  .ref-distante {
    color: var(--violet);
    border-color: var(--violet);
  }
  .ref-tag {
    color: var(--vert);
    border-color: var(--vert);
    background: var(--vert-fond);
  }
  .sujet {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .auteur,
  .date {
    color: var(--texte-2);
    white-space: nowrap;
    font-size: 12px;
  }
  .hash {
    color: var(--texte-2);
    font-size: 12px;
  }
  .plus {
    width: 100%;
    padding: 8px;
    border: none;
    background: var(--fond-2);
    color: var(--accent);
    cursor: pointer;
    font: inherit;
  }
  .detail-commit {
    position: sticky;
    top: 100px;
    max-height: calc(100vh - 120px);
    overflow: auto;
    padding: 10px 14px;
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    background: var(--fond);
  }
  .detail-commit .fermer {
    float: right;
    border: none;
    background: none;
    font-size: 20px;
    cursor: pointer;
    color: var(--texte-2);
  }
  .message {
    white-space: pre-wrap;
    font-family: var(--police);
    font-size: 13.5px;
    background: var(--fond-2);
    padding: 8px 10px;
    border-radius: var(--rayon);
  }
  .fichiers-commit {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .fichiers-commit li {
    display: flex;
    align-items: center;
  }
  .fichiers-commit button {
    flex: 1;
    min-width: 0;
    display: flex;
    gap: 6px;
    align-items: center;
    padding: 3px 4px;
    border: none;
    background: none;
    color: var(--texte);
    font: inherit;
    font-size: 12.5px;
    text-align: left;
    cursor: pointer;
    border-radius: 4px;
  }
  .fichiers-commit button:hover,
  .fichiers-commit button.actif {
    background: var(--fond-3);
  }
  .chemin {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--mono);
  }
  .statut-fichier {
    width: 16px;
    text-align: center;
    font-weight: 700;
    font-family: var(--mono);
  }
  .s-A {
    color: var(--vert);
  }
  .s-D {
    color: var(--rouge);
  }
  .s-M {
    color: var(--orange);
  }
  .s-R {
    color: var(--violet);
  }
  .ajouts {
    color: var(--vert);
    font-size: 11.5px;
  }
  .retraits {
    color: var(--rouge);
    font-size: 11.5px;
  }
  .wiki {
    padding: 0 6px;
    font-size: 12px;
  }
  .diff {
    font-size: 12px;
    line-height: 1.45;
    overflow: auto;
    background: var(--fond-2);
    padding: 8px 0;
    border-radius: var(--rayon);
  }
  .diff span {
    display: block;
    padding: 0 10px;
    white-space: pre;
  }
  .diff .ajout {
    background: color-mix(in srgb, var(--vert) 14%, transparent);
  }
  .diff .retrait {
    background: color-mix(in srgb, var(--rouge) 14%, transparent);
  }
  .diff .bloc {
    color: var(--cyan);
  }
  .diff .fichier {
    color: var(--texte-2);
    font-weight: 600;
  }
  @media (max-width: 900px) {
    .git.avec-detail {
      grid-template-columns: minmax(0, 1fr);
    }
    .auteur,
    .date {
      display: none;
    }
  }
</style>
