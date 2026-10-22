<script lang="ts">
  // Panneau latéral de la section active (EF-131, EF-140) : arbre des fichiers (Code), documents par
  // type (Projet) : la gestion en tête (backlog, ce qui attend une décision, favoris : EF-157), les plans,
  // les clarifications par sujet, puis un groupe par type avec ⚙ (skill de structure) et, en orange, ce
  // qui attend une relecture, compté jusqu'à l'accordéon ; les besoins en cascade avec leurs exigences et
  // leurs tests (EF-160) ; parcours avec leurs étapes et
  // « + » pour en ajouter, artefacts (Design thinking), arborescence des pages et remarques (Réception).
  // Le titre mène à l'accueil de la section ; un bouton replie tout puis, pressé à nouveau, déplie tout ;
  // hors du code, les accordéons sont fermés à l'arrivée (EF-150) et seul celui de l'élément ouvert se
  // déplie, centré.
  // @relu 2026-10-19 eb4d041f
  import type { Projet } from "../types/Projet";
  import type { DocumentResume } from "../types/DocumentResume";
  import { routeur, url, estCourant } from "../lib/routeur.svelte";
  import { NOMS, type Section } from "../lib/sections.svelte";
  import { atelier, ETATS, ecrire, sujets } from "../lib/atelier.svelte";
  import { aller } from "../lib/routeur.svelte";
  import { reception, remarquesDe, arbrePages, visible, type Noeud } from "../lib/reception.svelte";
  import { statique } from "../lib/statique";
  import { tick } from "svelte";
  import { api } from "../lib/api";
  import { aRelire as aRelireDe, tachesBacklog, favoris, basculerFavori, edition } from "../lib/edition.svelte";

  let { projet, section }: { projet: Projet; section: Section } = $props();
  // Parcours et clarifications ont leur arbre : ils ne sont pas répétés dans les groupes de documents.
  const docsDe = (zone: string) => projet.documents.filter((d) => d.zone === zone && (zone !== "atelier" || !["parcours", "clarification"].includes(d.genre)));

  // La gestion vient en tête du volet Projet (REC-017.2, REC-051) ; les plans ont leur groupe.
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

  const dossiers = $derived.by(() => {
    const m = new Map<string, typeof projet.fichiers>();
    for (const f of projet.fichiers) {
      const d = f.chemin.includes("/") ? f.chemin.slice(0, f.chemin.lastIndexOf("/")) : ".";
      if (!m.has(d)) m.set(d, []);
      m.get(d)!.push(f);
    }
    return [...m.entries()];
  });

  const grouper = (docs: DocumentResume[]) => {
    const m = new Map<string, { rang: number; docs: DocumentResume[] }>();
    for (const d of docs) {
      const [rang, nom] = ORDRE[d.genre] ?? [9, "Autres"];
      if (!m.has(nom)) m.set(nom, { rang, docs: [] });
      m.get(nom)!.docs.push(d);
    }
    return [...m.entries()].sort((a, b) => a[1].rang - b[1].rang);
  };
  const groupes = $derived(grouper(docsDe(section === "atelier" ? "atelier" : section === "reception" ? "reception" : "projet")));
  const ACCUEILS: Partial<Record<Section, () => string>> = { produit: url.produit, projet: url.projet, atelier: () => url.atelier(atelier.parcours), reception: url.reception };
  const titreDoc = (id: string) => projet.documents.find((d) => d.id === id)?.titre ?? "";
  const groupesClar = $derived(atelier.etat ? sujets(titreDoc) : []);
  const aRelire = $derived(aRelireDe(projet.documents));
  const ouvertes = $derived(atelier.etat?.clarifications.filter((c) => c.frontmatter.etat === "ouverte").length ?? 0);

  // Tâches du backlog à la charge de l'utilisateur (REC-048), relues avec le projet.
  const backlog = $derived(projet.documents.find((d) => d.genre === "backlog"));
  let taches = $state<string[]>([]);
  $effect(() => {
    edition.revision;
    if (section !== "projet" || !backlog) return;
    api
      .document(backlog.id)
      .then((d) => (taches = tachesBacklog(d.corps)))
      .catch(() => (taches = []));
  });
  const enAttente = $derived(taches.length + aRelire.length + ouvertes);

  // ⚙ d'un groupe : le skill qui dit comment ses documents sont structurés (REC-054).
  const SKILL_DE: Record<string, string> = {
    BDR: "wikik-structure-bdr",
    "Cahier des charges": "wikik-structure-cahier-des-charges",
    Besoins: "wikik-structure-besoin",
    Exigences: "wikik-structure-exigence",
    "Exigences non fonctionnelles": "wikik-structure-exigence",
    "Décisions (ADR)": "wikik-structure-adr",
    Specs: "wikik-structure-spec",
    Tests: "wikik-structure-test",
  };
  const aRelireIds = $derived(new Set(aRelire.map((d) => d.id)));

  // « + » du groupe Parcours (REC-039) : titre et version du nouveau parcours.
  let ajout = $state(false);
  let titreParcours = $state("");
  let versionParcours = $state("");
  let erreurParcours = $state("");
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

  // Lire la route rend l'état « actif » réactif : le panneau n'est plus recréé à chaque page, les
  // accordéons gardent leur état.
  const actif = (href: string) => (routeur.route, estCourant(href));
  const cleCourante = $derived(routeur.route.page === "reception-page" ? routeur.route.cle : null);
  const arbre = $derived(reception.etat ? arbrePages(reception.etat.pages.filter(visible)) : []);

  let panneau: HTMLElement;
  // Le bouton alterne : replier tout (REC-008, REC-010), puis tout déplier (REC-046).
  let replie = $state(true);
  $effect(() => {
    replie = section !== "produit";
  });
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
  {#if section === "projet" && atelier.etat}
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
      <li>
        <a {href} class:actif={cleCourante === p.cle} title={p.route ?? p.nom}>
          {p.nom}
          {#if n}<span class="compteur faible">{n}</span>{/if}
        </a>
        {#if enfants.length}{@render branche(enfants)}{/if}
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

  {#if section === "produit"}
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
          Parcours <small>{atelier.etat.tous_parcours.length}</small>
          <button class="ajouter" title="Nouveau parcours" aria-label="Nouveau parcours" onclick={(e) => { e.preventDefault(); ajout = !ajout; }}>＋</button>
        </summary>
        {#if ajout}
          <form class="ajout-parcours" onsubmit={creerParcours}>
            <input bind:value={titreParcours} placeholder="Titre du parcours" aria-label="Titre du parcours" />
            <input bind:value={versionParcours} placeholder="Version (v2…)" aria-label="Version majeure documentée" />
            <button class="principal" type="submit" disabled={!titreParcours.trim()}>Créer</button>
            {#if erreurParcours}<p class="erreur">{erreurParcours}</p>{/if}
          </form>
        {/if}
        <ul>
          {#each atelier.etat.tous_parcours as pc (pc.id)}
            {@const courant = pc.id === atelier.parcours}
            <li>
              <a href={url.atelier(pc.id)} class:actif={courant && routeur.route.page === "atelier"} title="{pc.id} · version {pc.version}">
                {pc.titre} <span class="compteur">{pc.faites}/{pc.total}</span>
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

    {#if section === "reception" && reception.etat}
      <ul class="liens-section">
        <li><a href={url.traitement()} class:actif={actif(url.traitement())}>Traitement des remarques</a></li>
      </ul>
      <details class="arbre">
        <summary>Pages de l'application <small>{reception.etat.pages.filter(visible).length}</small></summary>
        {@render branche(arbre)}
      </details>
    {/if}
    {#if section === "projet" && !groupes.some(([nom]) => nom === "Gestion")}{@render gestion([])}{/if}
    {#if !groupes.some(([nom]) => nom === "Plans" || nom === "Gestion")}{@render clarifications()}{/if}
    {#each groupes as [nom, g]}
      {#if nom === "Gestion" && section === "projet"}
        {@render gestion(g.docs)}
      {:else}
      {@const attente = g.docs.filter((d) => aRelireIds.has(d.id)).length}
      <details>
        <summary>
          {nom} <small>{g.docs.length}</small>
          {#if attente}<span class="compteur faible" title="{attente} à relire">{attente}</span>{/if}
          {#if section === "projet" && SKILL_DE[nom] && !statique}<a class="reglage" href={url.parametres() + "#skill-" + SKILL_DE[nom]} title="Comment sont structurés ces documents (skill {SKILL_DE[nom]})" aria-label="Structure de la section">⚙</a>{/if}
        </summary>
        <ul>
          {#if nom === "Besoins" && section === "projet"}<li><a href={url.cascade()} class:actif={actif(url.cascade())} class="lien-cascade">Cascade : besoins → exigences → tests</a></li>{/if}
          {#each g.docs as d}
            {@const href = url.document(d.id)}
            <li>
              <a {href} class:actif={actif(href)} title={d.titre}>
                <span class="id">{d.id}</span> {d.titre === d.id ? "" : d.titre}
                {#if aRelireIds.has(d.id)}<span class="compteur faible" title="à relire">●</span>{/if}
              </a>
              {#if d.genre === "besoin"}
                {@const es = projet.documents.filter((e) => e.besoins.includes(d.id))}
                {#if es.length}
                  <ul>
                    {#each es as e (e.id)}
                      {@const he = url.document(e.id)}
                      {@const ts = projet.documents.filter((t) => t.genre === "test" && t.exigences.includes(e.id))}
                      <li>
                        <a href={he} class:actif={actif(he)} title={e.titre}><span class="id">{e.id}</span> {e.titre}{#if !ts.length}<span class="compteur" title="sans test">○</span>{/if}</a>
                        {#if ts.length}
                          <ul>{#each ts as t (t.id)}{@const ht = url.document(t.id)}<li><a href={ht} class:actif={actif(ht)} title={t.titre}><span class="id">{t.id}</span> {t.titre}<span class="compteur" class:faible={t.validation !== "utilisateur" && t.validation !== "modele"}>{t.validation === "utilisateur" || t.validation === "modele" ? "✓" : t.validation === "echec" ? "✗" : "…"}</span></a></li>{/each}</ul>
                        {/if}
                      </li>
                    {/each}
                  </ul>
                {/if}
              {/if}
            </li>
          {/each}
        </ul>
      </details>
      {/if}
      {#if nom === "Plans" || (nom === "Gestion" && !groupes.some(([n]) => n === "Plans"))}{@render clarifications()}{/if}
    {:else}
      {#if section === "projet"}<p class="discret">Aucun document : <code>wikik nouveau besoin "…"</code>, ou le Design thinking.</p>{/if}
    {/each}
  {/if}
</div>
