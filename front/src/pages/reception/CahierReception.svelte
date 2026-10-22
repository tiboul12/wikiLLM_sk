<script lang="ts">
  // Cahier de réception (SPEC-23 R5, EF-140) : « Traiter avec l'agent » (EF-148) ou à la main ; les pages
  // trouvées dans le code, groupées, chacune avec sa
  // carte de réception (ses remarques) et son lien dans l'application ; scanner à nouveau, chercher les
  // écrans manquants par le LLM (annulable, écrans ajoutés listés avec leur raison, EF-146), renommer,
  // masquer, regrouper, ajouter une route. L'adresse de l'application est déduite du projet
  // ou réglée dans les paramètres : pas de saisie ici (REC-029). Chaque correction a la couleur de la
  // version qui l'a publiée, avec sa légende ; les remarques traitées attendent ta validation (EF-165).
  // @relu 2026-10-19 619d237e
  import type { Page } from "../../types/Page";
  import type { Ajout } from "../../types/Ajout";
  import { annulerLLM } from "../../lib/atelier.svelte";
  import { reception, ecrireReception, remarquesDe, actuelles, lienApplication, visible, couleurCommit, arbrePages, type Noeud } from "../../lib/reception.svelte";
  import LegendeVersions from "../../composants/reception/LegendeVersions.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Code from "../../composants/reception/Code.svelte";
  import EnvoiAgent from "../../composants/reception/EnvoiAgent.svelte";

  /** État de la réception ; la page n'est affichée que lorsqu'il est chargé. */
  const etat = $derived(reception.etat!);
  /** Montre aussi les pages masquées, pour pouvoir les réafficher. */
  let voirMasquees = $state(false);
  /** Clé de la page dont le formulaire de modification est ouvert. */
  let edition = $state<string | null>(null);
  /** Nom de la page en cours de modification. */
  let nom = $state("");
  /** Groupe de la page en cours de modification. */
  let groupe = $state("");
  /** Message de réussite de la dernière action. */
  let message = $state("");
  /** Erreur de la dernière action. */
  let erreur = $state("");
  /** Écrans ajoutés par la recherche du LLM, chacun avec sa raison (EF-146) ; `null` tant qu'elle n'a pas tourné. */
  let ajouts = $state<Ajout[] | null>(null);

  /** Pages listées : visibles, ou toutes si l'on montre les masquées. */
  const pages = $derived(etat.pages.filter((p) => voirMasquees || visible(p)));
  /** Profondeur de chaque page dans l'arborescence des routes, pour son retrait (REC-074, REC-075). */
  const profondeurs = new Map<string, number>();
  /**
   * Pages rangées comme l'application : « Général », l'accueil, puis un groupe par page de tête avec ses sous-pages
   * (`/atelier` → ses étapes ; `/clarifications` → une clarification) ; les groupes vides sont retirés.
   */
  const groupes = $derived.by(() => {
    const sortie: [string, Page[]][] = [["", pages.filter((p) => p.cle === "general")]];
    const aplatir = (n: Noeud, d: number, acc: Page[]) => {
      profondeurs.set(n.page.cle, d);
      acc.push(n.page);
      for (const e of n.enfants) aplatir(e, d + 1, acc);
      return acc;
    };
    const racines = arbrePages(pages.filter((p) => p.cle !== "general"));
    const accueil = racines.find((r) => r.page.route === "/");
    const tetes = accueil ? [...racines.filter((r) => r !== accueil), ...accueil.enfants] : racines;
    if (accueil) sortie.push([accueil.page.nom, [accueil.page]]);
    for (const t of tetes) sortie.push([t.page.nom, aplatir(t, 0, [])]);
    return sortie.filter(([, ps]) => ps.length);
  });
  /** Remarques ouvertes de la réception en cours. */
  const ouvertes = $derived(actuelles().filter((r) => r.statut === "ouverte").length);
  /** Toutes les remarques de la réception en cours. */
  const total = $derived(actuelles().length);
  /** Remarques traitées qui attendent la validation de l'utilisateur (EF-165). */
  const aValider = $derived(actuelles().filter((r) => r.statut === "traitee" && r.validation !== "validee").length);
  /** Nombre de pages masquées, rappelé sur la case qui les montre. */
  const masquees = $derived(etat.pages.filter((p) => !visible(p)).length);

  /**
   * Envoie une écriture de la réception et affiche son résultat. `completer` (recherche d'écrans par le LLM) occupe
   * l'écran pendant l'appel et rend la liste des ajouts ; une annulation n'est pas une erreur.
   */
  async function action(route: string, corps: unknown = {}, succes = "") {
    erreur = message = "";
    if (route === "completer") {
      reception.occupe = true;
      ajouts = null;
    }
    try {
      const r = await ecrireReception<unknown>(route, corps);
      if (route === "completer") ajouts = r as Ajout[];
      else message = succes;
      edition = null;
    } catch (e) {
      const m = String(e instanceof Error ? e.message : e);
      if (m.includes("annulé")) message = "Recherche annulée : la carte n'a pas changé.";
      else erreur = m;
    } finally {
      reception.occupe = false;
    }
  }

  /** Route de la page en cours de modification. */
  let route = $state("");
  /** Ouvre le formulaire de modification d'une page, prérempli. */
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
  <a class="tuile lien-tuile" class:alerte={aValider > 0} href={url.traitement()}><span class="valeur">{aValider}</span><span class="nom">à valider par toi →</span></a>
</div>
<LegendeVersions />

<div class="actions">
  <EnvoiAgent />
  <a class="bouton" href={url.traitement()}>Traiter à la main →</a>
  <button onclick={() => action("scanner", {}, "Carte mise à jour.")}>Scanner à nouveau</button>
  {#if reception.occupe}
    <button class="danger" onclick={annulerLLM}>Annuler la recherche</button>
  {:else}
    <button onclick={() => action("completer")} title="Le LLM ({etat.fournisseur}) lit le projet et ajoute les écrans que le scan n'a pas trouvés">Chercher les écrans manquants (LLM)</button>
  {/if}
  {#if masquees}<label class="discret"><input type="checkbox" bind:checked={voirMasquees} /> afficher les {masquees} page(s) masquée(s) ou disparue(s) du code</label>{/if}
  {#if message}<span class="succes">{message}</span>{/if}
</div>
<p class="discret aide-completer">
  « Scanner à nouveau » relit le code (routes, composants de pages). « Chercher les écrans manquants » demande au LLM les écrans sans
  route que le scan ne voit pas (vues par état, onglets, fenêtres) ; il n'ajoute que ceux dont il cite le composant, et dit pourquoi.
</p>
{#if erreur}<p class="erreur">{erreur}</p>{/if}
{#if ajouts}
  <section class="ajouts-llm">
    {#if ajouts.length}
      <strong>{ajouts.length} écran(s) ajouté(s) par le LLM :</strong>
      <ul>{#each ajouts as a (a.page.cle)}<li><a href={url.pageReception(a.page.cle)}>{a.page.nom}</a> {#if a.page.route}<code>{a.page.route}</code>{/if} : {a.raison}</li>{/each}</ul>
    {:else}
      <strong>Rien ne manque : le LLM n'a trouvé aucun écran de plus.</strong>
    {/if}
    <button class="discret-bouton" onclick={() => (ajouts = null)}>fermer</button>
  </section>
{/if}

{#each groupes as [g, ps] (g)}
  {#if g}<h2>{g}</h2>{/if}
  <ul class="pages-reception">
    {#each ps as p (p.cle)}
      {@const rs = remarquesDe(p.cle)}
      {@const ouv = rs.filter((r) => r.statut === "ouverte").length}
      <li class="page-reception" class:masquee={p.masque} class:disparue={p.disparue} style="margin-left: {(profondeurs.get(p.cle) ?? 0) * 22}px">
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
            {#each rs.filter((r) => r.statut !== "ouverte") as r (r.id)}
              {@const c = couleurCommit(r.commit)}
              <span class="point-correction" class:validee={r.validation === "validee"} style="--teinte: {c?.couleur ?? 'var(--gris)'}" title="{r.id} · {c?.libelle ?? r.statut}{r.validation === 'validee' ? ' · validée' : ' · à valider'}"></span>
            {/each}
          </span>
          {#if p.cle !== "general"}<button class="discret-bouton editer" onclick={() => editer(p)} title="Renommer, regrouper, masquer">✎</button>{/if}
        {/if}
      </li>
    {/each}
  </ul>
{/each}

<style>
  .point-correction {
    display: inline-block;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    border: 2px solid var(--teinte);
    background: transparent;
  }
  .point-correction.validee {
    background: var(--teinte);
  }
</style>
