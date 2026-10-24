<script lang="ts">
  // Panneau latéral de la section active (EF-131, EF-140). Code : l'arbre des fichiers et le graphe.
  // Projet : la gestion en tête (backlog, ce qui attend une décision, favoris : EF-157), puis l'arbre des
  // versions (ArbreVersions, EF-174, ADR-0018 : de la demande d'évolution aux tests) et les documents
  // transverses ; en orange, ce qui attend une relecture, compté jusqu'au nœud. Design thinking :
  // les ateliers (numéro, statut, étapes, « + ») puis, à part, les clarifications par sujet (EF-161) et
  // les artefacts. Git : l'historique, les branches et les tags (EF-166). Réception : l'arborescence des pages et les remarques, chacune avec la couleur de la
  // version qui a publié sa correction (EF-165).
  // Le titre mène à l'accueil de la section ; un bouton replie tout puis, pressé à nouveau, déplie tout ;
  // hors du code, les accordéons sont fermés à l'arrivée (EF-150) et seul celui de l'élément ouvert se
  // déplie, centré. Les arborescences (pages de réception, cascade besoin → exigences → tests) se replient
  // nœud par nœud, fermées à l'arrivée (REC-102).
  // @relu 2026-10-24 4f06e7e1
  import type { Projet } from "../types/Projet";
  import type { DocumentResume } from "../types/DocumentResume";
  import { routeur, url, estCourant } from "../lib/routeur.svelte";
  import { NOMS, type Section } from "../lib/sections.svelte";
  import { atelier, ETATS, ecrire, sujets, STATUTS_ATELIER } from "../lib/atelier.svelte";
  import { aller } from "../lib/routeur.svelte";
  import { reception, remarquesDe, arbrePages, visible, type Noeud } from "../lib/reception.svelte";
  import { statique } from "../lib/statique";
  import { tick } from "svelte";
  import { api } from "../lib/api";
  import { aRelire as aRelireDe, tachesBacklog, favoris, basculerFavori, edition } from "../lib/edition.svelte";
  import { rangerParVersion } from "../lib/versions";
  import ArbreVersions from "./ArbreVersions.svelte";
  import PastilleVersion from "./reception/PastilleVersion.svelte";

  let { projet, section }: { projet: Projet; section: Section } = $props();
  /** Documents d'une zone pour les groupes du panneau ; parcours et clarifications ont leur propre arbre et n'y sont pas répétés. */
  const docsDe = (zone: string) => projet.documents.filter((d) => d.zone === zone && (zone !== "atelier" || !["parcours", "clarification"].includes(d.genre)));

  /**
   * Rang et nom de groupe de chaque genre de document : la gestion en tête du volet Projet (REC-017.2, REC-051),
   * les plans à part, puis l'ordre de la démarche ; l'atelier suit l'ordre du parcours (SPEC-17), la réception
   * vient après (SPEC-23). Un genre absent tombe dans « Autres ».
   */
  const ORDRE: Record<string, [number, string]> = {
    backlog: [-2, "Gestion"],
    plan: [-1, "Plans"],
    "bdr-analyse": [1, "BDR"],
    "bdr-definition": [1, "BDR"],
    "cahier-des-charges": [2, "Cahier des charges"],
    besoin: [3, "Besoins"],
    exigence: [4, "Exigences"],
    "exigence-nf": [5, "Exigences non fonctionnelles"],
    test: [5.5, "Tests"],
    adr: [6, "Décisions (ADR)"],
    spec: [7, "Specs"],
    // Atelier (SPEC-17), dans l'ordre du parcours.
    parcours: [20, "Parcours"],
    pitch: [21, "Pitch"],
    axe: [22, "Axes"],
    ecrit: [23, "Écrits libres"],
    unite: [24, "Unités de sens"],
    carte: [25, "Carte mentale"],
    probleme: [26, "Problèmes"],
    analogie: [27, "Analogies"],
    idee: [28, "Idées"],
    analyse: [29, "Analyses"],
    scamper: [30, "SCAMPER"],
    storyboard: [31, "Storyboards"],
    clarification: [32, "Clarifications"],
    // Cahier de réception (SPEC-23).
    "carte-reception": [40, "Carte des pages"],
    remarque: [41, "Remarques"],
  };

  /** Fichiers du code regroupés par dossier, pour l'arbre de la section Code. */
  const dossiers = $derived.by(() => {
    const m = new Map<string, typeof projet.fichiers>();
    for (const f of projet.fichiers) {
      const d = f.chemin.includes("/") ? f.chemin.slice(0, f.chemin.lastIndexOf("/")) : ".";
      if (!m.has(d)) m.set(d, []);
      m.get(d)!.push(f);
    }
    return [...m.entries()];
  });

  /** Range des documents en groupes nommés, triés selon `ORDRE`. */
  const grouper = (docs: DocumentResume[]) => {
    const m = new Map<string, { rang: number; docs: DocumentResume[] }>();
    for (const d of docs) {
      const [rang, nom] = ORDRE[d.genre] ?? [9, "Autres"];
      if (!m.has(nom)) m.set(nom, { rang, docs: [] });
      m.get(nom)!.docs.push(d);
    }
    return [...m.entries()].sort((a, b) => a[1].rang - b[1].rang);
  };
  /** Zone Projet rangée par version (EF-174, ADR-0018) : les versions, et ce qui n'en a pas. */
  const rangement = $derived(section === "projet" ? rangerParVersion(projet.documents) : null);
  /**
   * Groupes de documents de la zone qui correspond à la section affichée ; en Projet, seulement les documents
   * transverses (sans version), le reste étant dans l'arbre des versions.
   */
  const groupes = $derived(grouper(rangement ? rangement.transverses : docsDe(section === "atelier" ? "atelier" : "reception")));
  /** Accueil de chaque section, sur lequel mène le titre du panneau.
   * @relu 2026-10-24 09b47684
   */
  const ACCUEILS: Partial<Record<Section, () => string>> = { produit: url.produit, projet: url.projet, atelier: url.ateliers, reception: url.reception, git: () => url.git() };

  // Section Git (EF-166) : branches, branches distantes et tags ; un clic montre leur commit.
  /** Références git listées dans la section Git (EF-166) : branche courante, branches locales et distantes, tags. */
  type Refs = { branche: string; branches: string[]; distantes: string[]; tags: string[] };
  /** Références git, relues quand le journal d'édition change ; rien hors de la section Git ou hors d'un dépôt. */
  let refs = $state<Refs | null>(null);
  $effect(() => {
    edition.revision;
    if (section !== "git") return;
    fetch("/api/git/historique?limite=1")
      .then((r) => (r.ok ? r.json() : null))
      .then((h) => (refs = h))
      .catch(() => (refs = null));
  });
  /** Titre d'un document par identifiant, pour nommer les sujets des clarifications. */
  const titreDoc = (id: string) => projet.documents.find((d) => d.id === id)?.titre ?? "";
  /** Clarifications regroupées par sujet (EF-161). */
  const groupesClar = $derived(atelier.etat ? sujets(titreDoc) : []);
  /** Documents qui attendent une relecture, signalés en orange. */
  const aRelire = $derived(aRelireDe(projet.documents));
  /** Nombre de clarifications encore ouvertes, comptées dans « À traiter ». */
  const ouvertes = $derived(atelier.etat?.clarifications.filter((c) => c.frontmatter.etat === "ouverte").length ?? 0);

  /** Backlog du projet, d'où sont tirées les tâches à la charge de l'utilisateur (REC-048). */
  const backlog = $derived(projet.documents.find((d) => d.genre === "backlog"));
  /** Tâches cochables du backlog encore ouvertes, relues avec le projet. */
  let taches = $state<string[]>([]);
  $effect(() => {
    edition.revision;
    if (section !== "projet" || !backlog) return;
    api
      .document(backlog.id)
      .then((d) => (taches = tachesBacklog(d.corps)))
      .catch(() => (taches = []));
  });
  /** Remarques traitées que l'utilisateur n'a pas encore validées (EF-165). */
  const remarquesAValider = $derived((reception.etat?.remarques ?? []).filter((r) => r.statut === "traitee" && r.validation !== "validee").length);
  /** Total de ce qui attend l'utilisateur : tâches, relectures, clarifications, remarques ; affiché sur « À traiter ». */
  const enAttente = $derived(taches.length + aRelire.length + ouvertes + remarquesAValider);

  /** Identifiants à relire, pour colorer chaque lien du panneau sans reparcourir la liste. */
  const aRelireIds = $derived(new Set(aRelire.map((d) => d.id)));

  /** Formulaire « + » du groupe Parcours ouvert (REC-039). */
  let ajout = $state(false);
  /** Titre du nouveau parcours. */
  let titreParcours = $state("");
  /** Version visée par le nouveau parcours. */
  let versionParcours = $state("");
  /** Erreur de création du parcours. */
  let erreurParcours = $state("");
  /** Crée le parcours et l'ouvre ; le formulaire se referme et se vide. */
  async function creerParcours(e: SubmitEvent) {
    e.preventDefault();
    erreurParcours = "";
    try {
      const pid = await ecrire<string>("parcours", { titre: titreParcours, version: versionParcours });
      ajout = false;
      titreParcours = versionParcours = "";
      aller(url.atelier(pid));
    } catch (x) {
      erreurParcours = String(x instanceof Error ? x.message : x);
    }
  }

  /**
   * Lien de la page ouverte ? Lire la route rend le résultat réactif : le panneau n'est pas recréé à chaque page
   * et les accordéons gardent leur état.
   */
  const actif = (href: string) => (routeur.route, estCourant(href));
  /** Clé de la page de réception ouverte, pour surligner son nœud dans l'arborescence. */
  const cleCourante = $derived(routeur.route.page === "reception-page" ? routeur.route.cle : null);
  /** Arborescence des pages de réception visibles. */
  const arbre = $derived(reception.etat ? arbrePages(reception.etat.pages.filter(visible)) : []);

  /** Racine du panneau, où l'on cherche les accordéons à replier et le lien actif. */
  let panneau: HTMLElement;
  /** Prochaine action du bouton : replier tout (REC-008, REC-010) puis tout déplier (REC-046) ; replié à l'arrivée hors de Produit. */
  let replie = $state(true);
  $effect(() => {
    replie = section !== "produit";
  });
  /** Replie ou déplie tous les accordéons du panneau. */
  function basculer() {
    replie = !replie;
    panneau.querySelectorAll("details").forEach((d) => (d.open = !replie));
  }
  // L'élément de la page ouverte : ses accordéons s'ouvrent et il vient au centre du panneau (REC-037).
  $effect(() => {
    routeur.route;
    section;
    tick().then(() => {
      const a = panneau?.querySelector("a.actif");
      if (!a) return;
      for (let d = a.parentElement?.closest("details"); d; d = d.parentElement?.closest("details")) d.open = true;
      const boite = panneau.closest(".barre");
      const r = a.getBoundingClientRect();
      const b = boite?.getBoundingClientRect();
      if (b && (r.top < b.top + 40 || r.bottom > b.bottom - 40)) a.scrollIntoView({ block: "center" });
    });
  });
</script>

{#snippet clarifications()}
  {#if section === "atelier" && atelier.etat}
    <details class="arbre">
      <summary><a class="lien-groupe" href={url.clarifications()} title="Tableau de bord des clarifications">Clarifications</a> <small>{atelier.etat.clarifications.length}</small></summary>
      <ul>
        {#each groupesClar as g (g.sujet)}
          <li>
            <a href={url.clarifications() + "?sujet=" + encodeURIComponent(g.sujet)} title={g.titre}>
              <span class="id">{g.sujet}</span> {g.titre}
              <span class="compteur" class:faible={!g.clarifie}>{g.clarifie ? "✓" : g.clarifications.length}</span>
            </a>
            <ul>
              {#each g.clarifications as c (c.id)}
                {@const href = url.clarification(c.id)}
                <li><a {href} class:actif={actif(href)}><span class="id">{c.id}</span> {c.frontmatter.etat === "close" ? "close" : "en cours"}</a></li>
              {/each}
            </ul>
          </li>
        {:else}
          <li class="discret">Aucune : « clarifier » sur un besoin ou une exigence.</li>
        {/each}
      </ul>
    </details>
  {/if}
{/snippet}

{#snippet gestion(docs: DocumentResume[])}
  <details class="arbre">
    <summary>Gestion {#if enAttente}<span class="compteur faible" title="{enAttente} élément(s) attendent ta décision">{enAttente}</span>{/if}</summary>
    <ul>
      {#each docs as d (d.id)}
        {@const href = url.document(d.id)}
        <li><a {href} class:actif={actif(href)} title={d.titre}><span class="id">{d.id}</span> {d.titre === d.id ? "" : d.titre}{#if d.genre === "backlog" && taches.length}<span class="compteur faible" title="tâches à ta charge">{taches.length}</span>{/if}</a></li>
      {/each}
      <li>
        <details class="sous-groupe">
          <summary>À traiter {#if enAttente}<span class="compteur faible">{enAttente}</span>{/if}</summary>
          <ul>
            {#if backlog}<li><a href={url.document(backlog.id)} title={taches.join("\n")}>Tâches du backlog à ta charge <span class="compteur" class:faible={taches.length > 0}>{taches.length}</span></a></li>{/if}
            <li><a href={url.relecture()} class:actif={actif(url.relecture())}>Documents à relire <span class="compteur" class:faible={aRelire.length > 0}>{aRelire.length}</span></a></li>
            {#if atelier.etat}<li><a href={url.clarifications()}>Clarifications ouvertes <span class="compteur" class:faible={ouvertes > 0}>{ouvertes}</span></a></li>{/if}
            {#if reception.etat}<li><a href={url.traitement()}>Remarques à valider <span class="compteur" class:faible={remarquesAValider > 0}>{remarquesAValider}</span></a></li>{/if}
          </ul>
        </details>
      </li>
      <li>
        <details class="sous-groupe">
          <summary>Favoris <small>{favoris.liste.length}</small></summary>
          <ul>
            {#each favoris.liste as f (f.url)}
              <li class="favori"><a href={f.url} class:actif={actif(f.url)} title={f.titre}>★ {f.titre}</a><button class="retirer" title="Retirer des favoris" aria-label="Retirer des favoris" onclick={() => basculerFavori(f.url, f.titre)}>×</button></li>
            {:else}
              <li class="discret">Aucun : ☆ sur l'onglet d'une page pour l'ajouter.</li>
            {/each}
          </ul>
        </details>
      </li>
    </ul>
  </details>
{/snippet}

{#snippet branche(noeuds: Noeud[])}
  <ul>
    {#each noeuds as { page: p, enfants } (p.cle)}
      {@const href = url.pageReception(p.cle)}
      {@const n = remarquesDe(p.cle).filter((r) => r.statut === "ouverte").length}
      {#snippet lien()}
        <a {href} class:actif={cleCourante === p.cle} title={p.route ?? p.nom}>
          {p.nom}
          {#if n}<span class="compteur faible">{n}</span>{/if}
        </a>
      {/snippet}
      <li>
        {#if enfants.length}
          <details class="noeud"><summary>{@render lien()}</summary>{@render branche(enfants)}</details>
        {:else}
          {@render lien()}
        {/if}
      </li>
    {/each}
  </ul>
{/snippet}

<div bind:this={panneau}>
  <div class="entete-panneau">
    {#if ACCUEILS[section]}
      <a class="titre-panneau" href={ACCUEILS[section]!()} title="Accueil de la section">{NOMS[section]}</a>
    {:else}
      <span class="titre-panneau">{NOMS[section]}</span>
    {/if}
    <button class="replier" onclick={basculer} title={replie ? "Déplier tout" : "Replier tout"} aria-label={replie ? "Déplier tout" : "Replier tout"}>{replie ? "⊞" : "⊟"}</button>
  </div>

  {#if section === "git"}
    <ul class="liens-section"><li><a href={url.git()} class:actif={actif(url.git())}>Historique</a></li></ul>
    {#if refs}
      {#each [["Branches", refs.branches], ["Branches distantes", refs.distantes], ["Tags", refs.tags]] as [titre, liste] (titre)}
        <details open={titre !== "Tags"}>
          <summary>{titre} <small>{(liste as string[]).length}</small></summary>
          <ul>{#each liste as string[] as r (r)}<li><a href={url.git({ ref: r })} title={r === refs.branche ? `${r} (branche courante)` : r}>{r}</a></li>{/each}</ul>
        </details>
      {/each}
    {/if}
  {:else if section === "produit"}
    <ul class="liens-section">
      <li><a href={url.graphe()} class:actif={actif(url.graphe())}>Graphe du projet</a></li>
    </ul>
    {#each dossiers as [dossier, fichiers]}
      <details open>
        <summary>{dossier}/</summary>
        <ul>
          {#each fichiers as f}
            {@const href = url.fichier(f.chemin)}
            <li>
              <a {href} class:actif={actif(href)} title={f.role ?? ""}>
                {f.chemin.split("/").pop()}
                <span class="compteur" class:faible={f.symboles > 0 && f.documentes / f.symboles < 0.5}>{f.documentes}/{f.symboles}</span>
              </a>
            </li>
          {/each}
        </ul>
      </details>
    {/each}
  {:else}
    {#if section === "atelier" && atelier.etat}
      <details class="arbre">
        <summary>
          Ateliers <small>{atelier.etat.tous_parcours.length}</small>
          <button class="ajouter" title="Nouvel atelier (une session)" aria-label="Nouvel atelier" onclick={(e) => { e.preventDefault(); ajout = !ajout; }}>＋</button>
        </summary>
        {#if ajout}
          <form class="ajout-parcours" onsubmit={creerParcours}>
            <input bind:value={titreParcours} placeholder="Titre de l'atelier" aria-label="Titre de l'atelier" />
            <input bind:value={versionParcours} placeholder="Version (v2…)" aria-label="Version majeure documentée" />
            <button class="principal" type="submit" disabled={!titreParcours.trim()}>Créer</button>
            {#if erreurParcours}<p class="erreur">{erreurParcours}</p>{/if}
          </form>
        {/if}
        <ul>
          {#each atelier.etat.tous_parcours as pc (pc.id)}
            {@const courant = pc.id === atelier.parcours}
            <li>
              <a href={url.atelier(pc.id)} class:actif={courant && routeur.route.page === "atelier"} title="Atelier n° {pc.numero} · {pc.id} · version {pc.version} · {STATUTS_ATELIER[pc.statut] ?? pc.statut}">
                <span class="id">n° {pc.numero}</span> {pc.titre} <span class="statut-atelier statut-{pc.statut}" title={STATUTS_ATELIER[pc.statut] ?? pc.statut}></span> <span class="compteur">{pc.faites}/{pc.total}</span>
              </a>
              {#if courant}
                <ul>
                  {#each atelier.etat.etapes as e, i (e.fiche.id)}
                    {@const href = url.etape(e.fiche.id, pc.id)}
                    <li>
                      <a {href} class:actif={actif(href)} title={ETATS[e.etat] ?? e.etat}>
                        <span class="puce etat-{e.etat}">{e.etat === "faite" ? "✓" : i + 1}</span> {e.fiche.titre}
                      </a>
                    </li>
                  {/each}
                </ul>
              {/if}
            </li>
          {/each}
        </ul>
      </details>
    {/if}

    {#if section === "atelier"}{@render clarifications()}{/if}

    {#if section === "reception" && reception.etat}
      <ul class="liens-section">
        <li><a href={url.traitement()} class:actif={actif(url.traitement())}>Traitement des remarques</a></li>
      </ul>
      <details class="arbre">
        <summary>Pages de l'application <small>{reception.etat.pages.filter(visible).length}</small></summary>
        {@render branche(arbre)}
      </details>
    {/if}
    {#if rangement}
      {@render gestion(groupes.find(([nom]) => nom === "Gestion")?.[1].docs ?? [])}
      <details class="arbre" open>
        <summary>Versions <small>{rangement.versions.length}</small></summary>
        <ArbreVersions versions={rangement.versions} {aRelireIds} />
      </details>
      {#if groupes.some(([nom]) => nom !== "Gestion")}<p class="titre-transverse">Transverse</p>{/if}
    {/if}
    {#each groupes as [nom, g]}
      {#if nom === "Gestion" && section === "projet"}
        <!-- déjà rendue en tête, avant les versions -->
      {:else}
      {@const attente = g.docs.filter((d) => aRelireIds.has(d.id)).length}
      <details>
        <summary>
          {nom} <small>{g.docs.length}</small>
          {#if attente}<span class="compteur faible" title="{attente} à relire">{attente}</span>{/if}
        </summary>
        <ul>
          {#each g.docs as d}
            {@const href = url.document(d.id)}
            {@const es = d.genre === "besoin" ? projet.documents.filter((e) => e.besoins.includes(d.id)) : []}
            {#snippet lienDoc()}
              <a {href} class:actif={actif(href)} title={d.titre}>
                <span class="id">{d.id}</span> {d.titre === d.id ? "" : d.titre}
                {#if aRelireIds.has(d.id)}<span class="compteur faible" title="à relire">●</span>{/if}
                {#if d.genre === "remarque"}
                  {@const rq = reception.etat?.remarques.find((x) => x.id === d.id)}
                  {#if rq?.commit}<PastilleVersion commit={rq.commit} />{:else if rq?.statut === "ouverte"}<span class="compteur faible" title="ouverte">○</span>{/if}
                {/if}
              </a>
            {/snippet}
            <li>
              {#if es.length}
                <!-- Cascade repliable (REC-102) : besoin → exigences → tests. -->
                <details class="noeud">
                  <summary>{@render lienDoc()}</summary>
                  <ul>
                    {#each es as e (e.id)}
                      {@const he = url.document(e.id)}
                      {@const ts = projet.documents.filter((t) => t.genre === "test" && t.exigences.includes(e.id))}
                      {#snippet lienEx()}<a href={he} class:actif={actif(he)} title={e.titre}><span class="id">{e.id}</span> {e.titre}{#if !ts.length}<span class="compteur" title="sans test">○</span>{/if}</a>{/snippet}
                      <li>
                        {#if ts.length}
                          <details class="noeud">
                            <summary>{@render lienEx()}</summary>
                            <ul>{#each ts as t (t.id)}{@const ht = url.document(t.id)}<li><a href={ht} class:actif={actif(ht)} title={t.titre}><span class="id">{t.id}</span> {t.titre}<span class="compteur" class:faible={t.validation !== "utilisateur" && t.validation !== "modele"}>{t.validation === "utilisateur" || t.validation === "modele" ? "✓" : t.validation === "echec" ? "✗" : "…"}</span></a></li>{/each}</ul>
                          </details>
                        {:else}
                          {@render lienEx()}
                        {/if}
                      </li>
                    {/each}
                  </ul>
                </details>
              {:else}
                {@render lienDoc()}
              {/if}
            </li>
          {/each}
        </ul>
      </details>
      {/if}
    {:else}
      {#if section === "projet"}<p class="discret">Aucun document : <code>wikik nouveau besoin "…"</code>, ou le Design thinking.</p>{/if}
    {/each}
  {/if}
</div>
