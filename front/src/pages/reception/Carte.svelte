<script lang="ts">
  // Carte des pages du cahier de réception (SPEC-23 R5) : les pages trouvées dans le code, groupées, avec
  // leurs remarques ouvertes ; scanner à nouveau, compléter par le LLM, renommer, masquer, regrouper.
  import type { Page } from "../../types/Page";
  import { reception, ecrireReception, remarquesDe, SOURCES } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Code from "../../composants/reception/Code.svelte";

  const etat = $derived(reception.etat!);
  let voirMasquees = $state(false);
  let edition = $state<string | null>(null);
  let nom = $state("");
  let groupe = $state("");
  let message = $state("");
  let erreur = $state("");

  const pages = $derived(etat.pages.filter((p) => voirMasquees || !p.masque));
  const groupes = $derived.by(() => {
    const m = new Map<string, Page[]>();
    for (const p of pages) {
      const g = p.cle === "general" ? "" : (p.groupe ?? SOURCES[p.source] ?? p.source);
      if (!m.has(g)) m.set(g, []);
      m.get(g)!.push(p);
    }
    return [...m.entries()];
  });
  const ouvertes = $derived(etat.remarques.filter((r) => r.statut === "ouverte").length);
  const masquees = $derived(etat.pages.filter((p) => p.masque).length);

  async function action(route: string, corps: unknown = {}, succes = "") {
    erreur = message = "";
    if (route === "completer") reception.occupe = true;
    try {
      const r = await ecrireReception<unknown>(route, corps);
      message = succes || (Array.isArray(r) ? `${r.length} page(s)` : "");
      edition = null;
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    } finally {
      reception.occupe = false;
    }
  }

  function editer(p: Page) {
    edition = p.cle;
    nom = p.nom;
    groupe = p.groupe ?? "";
  }
</script>

<h1>Cahier de réception</h1>
<p class="discret">
  Les pages de l'application, trouvées dans son code. Ouvre une page pour y déposer tes remarques : une carte par remarque, captures comprises.
  Tout est écrit dans <code>docs/reception/</code>.
</p>

<div class="tuiles">
  <div class="tuile"><span class="valeur">{pages.length}</span><span class="nom">pages</span></div>
  <div class="tuile" class:alerte={ouvertes > 0}><span class="valeur">{ouvertes}</span><span class="nom">remarques ouvertes</span></div>
  <div class="tuile"><span class="valeur">{etat.remarques.length - ouvertes}</span><span class="nom">traitées, reportées ou rejetées</span></div>
</div>

<div class="actions">
  <a class="bouton principal" href={url.traitement()}>Traiter les remarques →</a>
  <button onclick={() => action("scanner", {}, "Carte mise à jour.")}>Scanner à nouveau</button>
  <button onclick={() => action("completer")} disabled={reception.occupe} title="Le LLM lit le projet et propose les écrans manquants ({etat.fournisseur})">
    {reception.occupe ? "Le LLM cherche…" : "Compléter avec le LLM"}
  </button>
  {#if masquees}<label class="discret"><input type="checkbox" bind:checked={voirMasquees} /> afficher les {masquees} page(s) masquée(s)</label>{/if}
  {#if message}<span class="succes">{message}</span>{/if}
</div>
{#if erreur}<p class="erreur">{erreur}</p>{/if}

{#each groupes as [g, ps] (g)}
  {#if g}<h2>{g.charAt(0).toUpperCase() + g.slice(1)}</h2>{/if}
  <ul class="pages-reception">
    {#each ps as p (p.cle)}
      {@const rs = remarquesDe(p.cle)}
      {@const ouv = rs.filter((r) => r.statut === "ouverte").length}
      <li class="page-reception" class:masquee={p.masque} class:disparue={p.disparue}>
        {#if edition === p.cle}
          <form class="formulaire" onsubmit={(e) => { e.preventDefault(); action("page", { cle: p.cle, nom, groupe }); }}>
            <label><span>Nom</span><input bind:value={nom} /></label>
            <label><span>Groupe</span><input bind:value={groupe} placeholder="(aucun)" /></label>
            <div class="actions">
              <button class="principal" type="submit">Enregistrer</button>
              <button type="button" onclick={() => action("page", { cle: p.cle, masque: !p.masque })}>{p.masque ? "Afficher" : "Masquer"}</button>
              <button type="button" class="discret-bouton" onclick={() => (edition = null)}>Annuler</button>
            </div>
          </form>
        {:else}
          <a class="titre-page" href={url.pageReception(p.cle)}>
            <strong>{p.nom}</strong>
            {#if p.route}<code>{p.route}</code>{/if}
          </a>
          <div class="meta-page">
            {#if p.symboles[0]}<Code id={p.symboles[0]} />{/if}
            {#each p.exigences as ex}<a class="pastille" href={url.document(ex)}>{ex}</a>{/each}
            {#if p.disparue}<span class="pastille statut-remplacé">plus dans le code</span>{/if}
          </div>
          <span class="compteurs">
            {#if ouv}<span class="pastille compte-ouvert">{ouv}</span>{/if}
            {#if rs.length - ouv}<span class="discret">{rs.length - ouv} ✓</span>{/if}
          </span>
          {#if p.cle !== "general"}<button class="discret-bouton editer" onclick={() => editer(p)} title="Renommer, regrouper, masquer">✎</button>{/if}
        {/if}
      </li>
    {/each}
  </ul>
{/each}
