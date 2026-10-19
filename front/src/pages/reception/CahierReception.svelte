<script lang="ts">
  // Cahier de réception (SPEC-23 R5, EF-140) : les pages trouvées dans le code, groupées, chacune avec sa
  // carte de réception (ses remarques) et son lien dans l'application ; scanner à nouveau, compléter par le
  // LLM, renommer, masquer, regrouper, ajouter une route. L'adresse de l'application est déduite du projet
  // ou réglée dans les paramètres : pas de saisie ici (REC-029).
  // @relu 2026-10-19 619d237e
  import type { Page } from "../../types/Page";
  import { reception, ecrireReception, remarquesDe, actuelles, lienApplication, visible, SOURCES } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Code from "../../composants/reception/Code.svelte";

  const etat = $derived(reception.etat!);
  let voirMasquees = $state(false);
  let edition = $state<string | null>(null);
  let nom = $state("");
  let groupe = $state("");
  let message = $state("");
  let erreur = $state("");

  const pages = $derived(etat.pages.filter((p) => voirMasquees || visible(p)));
  const groupes = $derived.by(() => {
    const m = new Map<string, Page[]>();
    for (const p of pages) {
      const g = p.cle === "general" ? "" : (p.groupe ?? SOURCES[p.source] ?? p.source);
      if (!m.has(g)) m.set(g, []);
      m.get(g)!.push(p);
    }
    return [...m.entries()];
  });
  const ouvertes = $derived(actuelles().filter((r) => r.statut === "ouverte").length);
  const total = $derived(actuelles().length);
  const masquees = $derived(etat.pages.filter((p) => !visible(p)).length);

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

  let route = $state("");
  function editer(p: Page) {
    edition = p.cle;
    nom = p.nom;
    groupe = p.groupe ?? "";
    route = p.route ?? "";
  }
</script>

<h1>Cahier de réception</h1>
<p class="discret">
  Les pages de l'application, trouvées dans son code. Chaque page a sa carte de réception : ouvre-la pour y déposer tes remarques, captures comprises.
  Tout est écrit dans <code>docs/reception/</code>.
</p>

<div class="tuiles">
  <div class="tuile"><span class="valeur">{pages.length}</span><span class="nom">pages</span></div>
  <div class="tuile" class:alerte={ouvertes > 0}><span class="valeur">{ouvertes}</span><span class="nom">remarques ouvertes</span></div>
  <div class="tuile"><span class="valeur">{total - ouvertes}</span><span class="nom">traitées, reportées ou rejetées</span></div>
</div>

<div class="actions">
  <a class="bouton principal" href={url.traitement()}>Traiter les remarques →</a>
  <button onclick={() => action("scanner", {}, "Carte mise à jour.")}>Scanner à nouveau</button>
  <button onclick={() => action("completer")} disabled={reception.occupe} title="Le LLM lit le projet et propose les écrans manquants ({etat.fournisseur})">
    {reception.occupe ? "Le LLM cherche…" : "Compléter avec le LLM"}
  </button>
  {#if masquees}<label class="discret"><input type="checkbox" bind:checked={voirMasquees} /> afficher les {masquees} page(s) masquée(s) ou disparue(s) du code</label>{/if}
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
          <form class="formulaire" onsubmit={(e) => { e.preventDefault(); action("page", { cle: p.cle, nom, groupe, route }); }}>
            <label><span>Nom</span><input bind:value={nom} /></label>
            <label><span>Route dans l'application</span><input bind:value={route} placeholder="/chemin (pour le lien « ouvrir »)" /></label>
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
          {#if lienApplication(p.route)}<a class="ouvrir-app" href={lienApplication(p.route)} target="_blank" rel="noopener" title="Ouvrir cette page dans l'application">↗</a>{/if}
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
