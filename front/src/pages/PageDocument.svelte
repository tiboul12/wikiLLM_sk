<script lang="ts">
  // Page document (EF-042, EF-143) : frontmatter, corps Markdown (images relatives servies par
  // `/api/image`), liens résolus, identifiants et chemins cités liés d'office, ce qui le cite ; liens vers
  // l'atelier pour un besoin, une exigence ou un artefact (SPEC-21), « Reprendre » pour une clarification.
  // Un plan ou le backlog s'ouvre en tableau à colonnes. Le propriétaire peut le valider ou l'éditer en
  // entier (EF-158) ; `?editer=1` ouvre directement l'éditeur (« Corriger » de la relecture). Une exigence
  // montre ses tests (« ＋ créer un test »), un test son ticket de validation (EF-160) ; un diagramme
  // Mermaid se modifie avec son aperçu (EF-167). L'encart ⚙ Structure montre le skill qui dit comment ce genre
  // de document est rédigé, et le corrige sur place (EF-176).
  // @ecrans /projet/:id Page d'un document
  // @relu 2026-10-24 b2590faa
  import { tick } from "svelte";
  import { api } from "../lib/api";
  import { rendre, rendreMermaid } from "../lib/markdown";
  import { brancherHooks, reference } from "../lib/hook";
  import { url } from "../lib/routeur.svelte";
  import type { PageDocument } from "../types/PageDocument";
  import Lien from "../composants/Lien.svelte";
  import { lienAtelier } from "../lib/atelier.svelte";
  import { autoLien } from "../lib/connus.svelte";
  import { colonnesPlan, colonnesBacklog } from "../lib/gestion";
  import Kanban from "../composants/Kanban.svelte";
  import Editeur from "../composants/Editeur.svelte";
  import EditeurMermaid from "../composants/atelier/EditeurMermaid.svelte";
  import { terminal } from "../lib/terminal.svelte";
  import { GENRES_A_RELIRE, changerStatut, signaler, VALIDATIONS, validerTest, nouveauDocument } from "../lib/edition.svelte";
  import { connus } from "../lib/connus.svelte";
  import { statique } from "../lib/statique";
  import PromptSysteme from "../composants/PromptSysteme.svelte";
  import { SKILL_STRUCTURE } from "../lib/edition.svelte";

  let { id, ancre }: { id: string; ancre: string } = $props();
  /** Document chargé (frontmatter, corps, liens résolus, rétroliens). */
  let page = $state<PageDocument | null>(null);
  /** Erreur de chargement, affichée à la place du document. */
  let erreur = $state("");
  /** Conteneur du corps rendu : on y branche les hooks de référence et on y rend les diagrammes Mermaid. */
  let corps = $state<HTMLElement | null>(null);

  /** Champs du frontmatter montrés dans l'en-tête, dans cet ordre ; les autres restent dans le détail repliable. */
  const CHAMPS = ["statut", "priorite", "version", "date", "besoins", "exigences", "decisions", "remplace", "remplace_par", "sources"];

  // Page recréée par {#key} à chaque changement de route : la valeur initiale suffit.
  // svelte-ignore state_referenced_locally
  api
    .document(id)
    .then((p) => (page = p))
    .catch((e) => (erreur = String(e)));

  /**
   * Corps rendu en HTML : liens Markdown remplacés par leur cible résolue par l'index (fichier ou document),
   * identifiants et chemins cités liés d'office.
   */
  const html = $derived.by(() => {
    if (!page) return "";
    const table = new Map(page.liens.map((l) => [l.brute, l.cible]));
    return rendre(
      page.corps,
      (brute) => {
        const c = table.get(brute);
        if (!c) return null;
        return c.fichier ? url.fichier(c.fichier, c.libelle) : url.document(c.id);
      },
      reference(page.id),
      page.chemin,
      autoLien,
    );
  });
  /** Colonnes du tableau d'un plan ou du backlog ; `null` pour les autres genres, rendus en document. */
  const colonnes = $derived(page?.genre === "plan" ? colonnesPlan(page.corps) : page?.genre === "backlog" ? colonnesBacklog(page.corps) : null);
  /** Vue d'un plan ou du backlog : tableau à colonnes ou document brut. */
  let vue = $state<"tableau" | "document">("tableau");
  /** Éditeur complet ouvert ; `?editer=1` l'ouvre d'office, jamais dans le site statique. */
  let edite = $state(!statique && new URLSearchParams(location.search).get("editer") === "1");
  /** Éditeur Mermaid ouvert sur les diagrammes du document. */
  let diagramme = $state(false);
  /** Document proposé ou brouillon d'un genre qui se relit : le bouton « Valider » s'affiche. */
  const aValider = $derived(page && GENRES_A_RELIRE.includes(page.genre) && (page.statut === "proposé" || page.statut === "brouillon"));
  /** Tests qui vérifient l'exigence affichée (cascade, EF-160). */
  const tests = $derived(page && (page.genre === "exigence" || page.genre === "exigence-nf") ? connus.tests.filter((t) => t.exigences.includes(page!.id)) : []);
  /** Ticket de validation du test affiché : libellé et couleur de son verdict. */
  const ticket = $derived(page?.genre === "test" ? (VALIDATIONS[String(page.frontmatter.validation ?? "a_valider")] ?? VALIDATIONS.a_valider) : null);
  /** Enregistre le verdict de l'utilisateur sur le test ; une erreur s'affiche dans le bandeau. */
  async function verdict(v: "utilisateur" | "echec" | "a_valider") {
    if (!page) return;
    try {
      await validerTest(page.id, v);
    } catch (e) {
      signaler(String(e instanceof Error ? e.message : e), true);
    }
  }
  /** Crée un test relié à l'exigence affichée, titré « Vérifier … », puis l'ouvre. */
  async function creerTest() {
    if (!page) return;
    try {
      await nouveauDocument("test", `Vérifier ${page.titre.charAt(0).toLowerCase()}${page.titre.slice(1)}`, [page.id]);
    } catch (e) {
      signaler(String(e instanceof Error ? e.message : e), true);
    }
  }
  /** Valide le document : « accepté » pour un ADR, « validé » pour les autres genres. */
  async function valider() {
    if (!page) return;
    try {
      await changerStatut(page.id, page.genre === "adr" ? "accepté" : "validé");
    } catch (e) {
      signaler(String(e instanceof Error ? e.message : e), true);
    }
  }

  $effect(() => {
    if (!corps || !html) return;
    brancherHooks(corps);
    tick().then(async () => {
      await rendreMermaid(corps!);
      if (ancre) document.getElementById(ancre)?.scrollIntoView({ block: "start" });
    });
  });

  /** Valeurs d'un champ de frontmatter sous forme de liste, qu'il soit scalaire ou tableau. */
  const valeurs = (v: unknown): string[] => (Array.isArray(v) ? v.map(String) : v === undefined || v === null ? [] : [String(v)]);
  /** Valeur qui ressemble à un identifiant de document plutôt qu'à un chemin de fichier. */
  const estId = (v: string) => !v.includes("/") && !v.endsWith(".md");
  /** Genres qui s'ouvrent dans l'atelier et peuvent y être clarifiés, en plus des artefacts (SPEC-21 R1). */
  const RAPPELABLES = ["besoin", "exigence", "exigence-nf"];
  /** Technique de l'atelier qui a produit l'artefact, si c'en est un. */
  const technique = $derived(typeof page?.frontmatter.technique === "string" ? page.frontmatter.technique : null);
  /** Lien « Ouvrir dans l'atelier » : l'étape de la technique, la clarification elle-même, ou l'accueil de l'atelier. */
  const ouvrir = $derived(page && (technique || RAPPELABLES.includes(page.genre)) ? lienAtelier(technique && technique !== "clarification" ? `/atelier/${technique}` : page.genre === "clarification" ? url.clarification(page.id) : "/atelier") : null);
  /** Lien « Clarifier » qui ouvre une clarification sur ce document ; absent pour une clarification. */
  const clarifier = $derived(page && page.genre !== "clarification" && (technique || RAPPELABLES.includes(page.genre)) ? lienAtelier(url.clarifier(page.id)) : null);
</script>

{#if erreur}
  <p class="erreur">{erreur}</p>
{:else if page}
  <p class="fil">{page.chemin}</p>
  <div class="frontmatter">
    <span class="genre">{page.genre}</span>
    {#each CHAMPS as champ}
      {#each valeurs(page.frontmatter[champ]) as v, i}
        {#if i === 0}<span class="cle">{champ}</span>{/if}
        {#if estId(v) && ["besoins", "exigences", "decisions", "remplace", "remplace_par", "sources"].includes(champ)}
          <a class="pastille" href={url.document(v)}>{v}</a>
        {:else}
          <span class="pastille {champ === 'statut' ? 'statut-' + v : ''}">{v}</span>
        {/if}
      {/each}
    {/each}
    <a class="action" href={url.graphe(page.id)} title="Graphe local">◎ graphe</a>
    {#if page.genre === "clarification" && lienAtelier("/")}<a class="bouton principal reprendre" href={url.clarification(page.id)}>Reprendre la clarification →</a>
    {:else if ouvrir}<a class="action atelier" href={ouvrir}>ouvrir dans le Design thinking</a>{/if}
    {#if clarifier}<a class="action atelier" href={clarifier}>clarifier</a>{/if}
    {#if terminal.proprietaire && !statique}
      <span class="gestes-document">
        {#if aValider}<button class="valider-doc" onclick={valider} title="Passer à « {page.genre === 'adr' ? 'accepté' : 'validé'} » (Ctrl+Z pour annuler)">✓ Valider</button>{/if}
        {#if !edite}<button class="editer-doc" onclick={() => (edite = true)} title="Modifier tout le document, frontmatter compris">✎ Éditer</button>{/if}
        {#if !diagramme && page.corps.includes("```mermaid")}<button class="editer-doc" onclick={() => (diagramme = true)} title="Modifier le diagramme avec son aperçu">✎ Diagramme</button>{/if}
      </span>
    {/if}
  </div>
  {#if SKILL_STRUCTURE[page.genre]}<PromptSysteme nom={SKILL_STRUCTURE[page.genre]} libelle="Structure" />{/if}
  {#if edite}<Editeur id={page.id} fermer={() => (edite = false)} />{/if}
  {#if diagramme}<EditeurMermaid id={page.id} fermer={() => (diagramme = false)} />{/if}
  {#if ticket}
    <div class="ticket-test ticket-{ticket[1]}">
      <span>Ticket de validation : <strong>{ticket[0]}</strong>{#if page.frontmatter.valide_le} · le {page.frontmatter.valide_le}{/if}</span>
      {#if terminal.proprietaire && !statique}
        <button onclick={() => verdict("utilisateur")} title="Tu as constaté l'attendu">✓ Je valide</button>
        <button onclick={() => verdict("echec")} title="L'attendu n'est pas constaté">✗ Échec</button>
        {#if page.frontmatter.validation && page.frontmatter.validation !== "a_valider"}<button class="discret-bouton" onclick={() => verdict("a_valider")}>Rouvrir</button>{/if}
      {/if}
    </div>
  {/if}
  {#if page.genre === "exigence" || page.genre === "exigence-nf"}
    <div class="tests-exigence">
      <strong>Tests</strong>
      {#each tests as t (t.id)}
        {@const [mot, classe] = VALIDATIONS[t.validation ?? "a_valider"] ?? VALIDATIONS.a_valider}
        <a class="pastille" href={url.document(t.id)} title={t.titre}>{t.id} · <span class="ticket-{classe}">{mot}</span></a>
      {:else}
        <span class="discret">aucun test : la cascade s'arrête ici</span>
      {/each}
      {#if terminal.proprietaire && !statique}<button class="discret-bouton" onclick={creerTest}>＋ créer un test</button>{/if}
    </div>
  {/if}
  {#if colonnes}
    <div class="bascule vue-gestion" role="group" aria-label="Affichage">
      <button class:actif={vue === "tableau"} onclick={() => (vue = "tableau")}>Tableau</button>
      <button class:actif={vue === "document"} onclick={() => (vue = "document")}>Document</button>
    </div>
    {#if vue === "tableau"}<Kanban {colonnes} resoudre={autoLien} />{/if}
  {/if}
  <article class="prose document" class:cache={edite || (colonnes && vue === "tableau")} bind:this={corps}>{@html html}</article>
  {#if page.cite_par.length || page.references.length}
    <section class="cite-par">
      {#if page.cite_par.length}
        <h2>Cité dans le code</h2>
        <ul>{#each page.cite_par as l}<li><span class="etiquette">@{l.genre}</span> <Lien lien={l} /></li>{/each}</ul>
      {/if}
      {#if page.references.length}
        <h2>Documents qui y renvoient</h2>
        <ul>{#each page.references as l}<li><Lien lien={l} /></li>{/each}</ul>
      {/if}
    </section>
  {/if}
{:else}
  <p class="discret">Chargement…</p>
{/if}

<style>
  .ticket-test,
  .tests-exigence {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    padding: 8px 12px;
    margin: 10px 0;
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    background: var(--fond-2);
    font-size: 14px;
  }
  .ticket-test button {
    padding: 3px 10px;
    border: 1px solid var(--bord-fort);
    border-radius: var(--rayon);
    background: var(--fond);
    color: var(--texte);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .ticket-test.ticket-ok {
    border-color: var(--vert);
    background: var(--vert-fond);
  }
  .ticket-test.ticket-ko {
    border-color: var(--rouge);
    background: var(--rouge-fond);
  }
  .ticket-ok {
    color: var(--vert);
  }
  .ticket-ko {
    color: var(--rouge);
  }
  .ticket-attente {
    color: var(--orange);
  }
  .gestes-document {
    display: inline-flex;
    gap: 6px;
    margin-left: auto;
  }
  .valider-doc,
  .editer-doc {
    padding: 3px 10px;
    border-radius: var(--rayon);
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .valider-doc {
    border: 1px solid var(--vert);
    background: var(--vert-fond);
    color: var(--vert);
  }
  .editer-doc {
    border: 1px solid var(--bord-fort);
    background: var(--fond-2);
    color: var(--texte);
  }
</style>
