<script lang="ts">
  // Cascade du projet (EF-160, REC-055) : chaque besoin, les exigences qui le couvrent (avec le cahier des
  // charges de leur version), et pour chaque exigence ses tests et leur ticket de validation (à valider,
  // validé par toi ou par le modèle, échec). Les trous se voient : besoin sans exigence, exigence sans
  // test. Filtre par version.
  import type { Projet } from "../types/Projet";
  import type { DocumentResume } from "../types/DocumentResume";
  import { url } from "../lib/routeur.svelte";
  import { VALIDATIONS, nouveauDocument, signaler } from "../lib/edition.svelte";
  import { terminal } from "../lib/terminal.svelte";
  import { statique } from "../lib/statique";

  let { projet }: { projet: Projet | null } = $props();
  /** Documents de la zone Projet, seuls concernés par la cascade. */
  const docs = $derived((projet?.documents ?? []).filter((d) => d.zone === "projet"));
  /** Besoins du projet. */
  const besoins = $derived(docs.filter((d) => d.genre === "besoin"));
  /** Exigences fonctionnelles et non fonctionnelles. */
  const exigences = $derived(docs.filter((d) => d.genre === "exigence" || d.genre === "exigence-nf"));
  /** Tests (`TEST-xxx`), reliés aux exigences qu'ils vérifient. */
  const tests = $derived(docs.filter((d) => d.genre === "test"));
  /** Cahier des charges de chaque version, pour le rappeler à côté des exigences de cette version. */
  const cdc = $derived(new Map(docs.filter((d) => d.genre === "cahier-des-charges").map((d) => [d.version ?? "", d])));
  /** Versions qui ont des exigences, de la plus récente à la plus ancienne (tri numérique : v10 après v9). */
  const versions = $derived([...new Set(exigences.map((e) => e.version ?? ""))].filter(Boolean).sort((a, b) => b.localeCompare(a, "fr", { numeric: true })));
  /** Version filtrée ; vide pour toutes. */
  let version = $state("");

  /** Exigences qui couvrent un besoin, dans la version filtrée. */
  const exigencesDe = (b: string) => exigences.filter((e) => e.besoins.includes(b) && (!version || e.version === version));
  /** Tests qui vérifient une exigence. */
  const testsDe = (e: string) => tests.filter((t) => t.exigences.includes(e));
  /** Exigences qui ne citent aucun besoin existant : montrées à part pour que le trou se voie. */
  const orphelines = $derived(exigences.filter((e) => !e.besoins.some((b) => besoins.some((x) => x.id === b)) && (!version || e.version === version)));
  /** Besoins affichés : tous, ou ceux qui ont une exigence dans la version ou en relèvent. */
  const visibles = $derived(besoins.filter((b) => !version || exigencesDe(b.id).length || b.version === version));

  /** Ticket de validation d'un test : libellé et couleur de son verdict. */
  const ticket = (t: DocumentResume) => VALIDATIONS[t.validation ?? "a_valider"] ?? VALIDATIONS.a_valider;
  /** Exigences de la version qui ont au moins un test. */
  const couvertes = $derived(exigences.filter((e) => (!version || e.version === version) && testsDe(e.id).length).length);
  /** Exigences de la version, base du taux de couverture. */
  const total = $derived(exigences.filter((e) => !version || e.version === version).length);
  /** Tests validés, par l'utilisateur ou par le modèle. */
  const validees = $derived(tests.filter((t) => t.validation === "utilisateur" || t.validation === "modele").length);

  /** Crée un test relié à l'exigence, titré « Vérifier … » ; une erreur s'affiche dans le bandeau. */
  async function creerTest(e: DocumentResume) {
    try {
      await nouveauDocument("test", `Vérifier ${e.titre.charAt(0).toLowerCase()}${e.titre.slice(1)}`, [e.id]);
    } catch (x) {
      signaler(String(x instanceof Error ? x.message : x), true);
    }
  }
</script>

{#snippet ligneExigence(e: DocumentResume)}
  {@const ts = testsDe(e.id)}
  <li class="exigence">
    <a href={url.document(e.id)}><span class="id">{e.id}</span> {e.titre}</a>
    {#if e.version && cdc.get(e.version)}<a class="pastille" href={url.document(cdc.get(e.version)!.id)} title="Cahier des charges qui porte l'exigence">{cdc.get(e.version)!.id}</a>{/if}
    <span class="pastille statut-{e.statut}">{e.statut}</span>
    {#if ts.length}
      <ul class="tests">
        {#each ts as t (t.id)}
          {@const [mot, classe] = ticket(t)}
          <li><a href={url.document(t.id)}><span class="id">{t.id}</span> {t.titre}</a> <span class="ticket {classe}">{mot}</span></li>
        {/each}
      </ul>
    {:else}
      <span class="manque">sans test</span>
      {#if terminal.proprietaire && !statique}<button class="discret-bouton" onclick={() => creerTest(e)}>＋ test</button>{/if}
    {/if}
  </li>
{/snippet}

<p class="fil"><a href={url.projet()}>Projet</a> · cascade</p>
<h1>Cascade</h1>
{#if !projet}
  <p class="discret">Chargement…</p>
{:else}
  <p class="discret">Besoin → cahier des charges → exigences → un test par exigence → ticket validé par toi ou par le modèle (outil MCP <code>test_valider</code>). Un test se crée d'un clic depuis une exigence qui n'en a pas.</p>
  <div class="tuiles">
    <div class="tuile"><span class="valeur">{besoins.length}</span><span class="nom">besoins</span></div>
    <div class="tuile"><span class="valeur">{total}</span><span class="nom">exigences</span></div>
    <div class="tuile" class:alerte={couvertes < total}><span class="valeur">{couvertes} / {total}</span><span class="nom">exigences testées</span></div>
    <div class="tuile"><span class="valeur">{validees} / {tests.length}</span><span class="nom">tests validés</span></div>
  </div>
  <div class="actions">
    <label>Version <select bind:value={version}><option value="">toutes</option>{#each versions as v}<option value={v}>{v}</option>{/each}</select></label>
  </div>
  <ul class="cascade">
    {#each visibles as b (b.id)}
      {@const es = exigencesDe(b.id)}
      <li class="besoin">
        <a href={url.document(b.id)}><span class="id">{b.id}</span> <strong>{b.titre}</strong></a>
        <span class="pastille statut-{b.statut}">{b.statut}</span>
        {#if es.length}<ul>{#each es as e (e.id)}{@render ligneExigence(e)}{/each}</ul>{:else}<span class="manque">aucune exigence</span>{/if}
      </li>
    {/each}
    {#if orphelines.length}
      <li class="besoin">
        <strong>Exigences sans besoin connu</strong>
        <ul>{#each orphelines as e (e.id)}{@render ligneExigence(e)}{/each}</ul>
      </li>
    {/if}
  </ul>
{/if}

<style>
  .cascade,
  .cascade ul {
    list-style: none;
    padding-left: 0;
  }
  .cascade ul {
    padding-left: 22px;
    border-left: 2px solid var(--bord);
    margin: 6px 0 6px 6px;
  }
  .besoin {
    margin: 0 0 16px;
  }
  .exigence {
    margin: 4px 0;
  }
  .tests li {
    font-size: 13.5px;
  }
  .manque {
    color: var(--orange);
    font-size: 12.5px;
    margin-left: 6px;
  }
  .ticket {
    font-size: 11.5px;
    padding: 1px 7px;
    border-radius: 999px;
    margin-left: 4px;
  }
  .ticket.attente {
    background: var(--orange-fond);
    color: var(--orange);
  }
  .ticket.ok {
    background: var(--vert-fond);
    color: var(--vert);
  }
  .ticket.ko {
    background: var(--rouge-fond);
    color: var(--rouge);
  }
</style>
