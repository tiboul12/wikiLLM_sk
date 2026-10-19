<script lang="ts">
  // Panneau latéral de la section active (EF-131, EF-140) : arbre des fichiers (Produit), documents par
  // type, la gestion en tête puis les clarifications par sujet (Projet), parcours avec leurs étapes et
  // « + » pour en ajouter, artefacts (Design thinking), arborescence des pages et remarques (Réception).
  // Le titre mène à l'accueil de la section ; un bouton replie tout ; l'élément ouvert est déplié et centré.
  import type { Projet } from "../types/Projet";
  import type { DocumentResume } from "../types/DocumentResume";
  import { routeur, url, estCourant } from "../lib/routeur.svelte";
  import { NOMS, type Section } from "../lib/sections.svelte";
  import { atelier, ETATS, ecrire, sujets } from "../lib/atelier.svelte";
  import { aller } from "../lib/routeur.svelte";
  import { reception, remarquesDe, arbrePages, visible, type Noeud } from "../lib/reception.svelte";
  import { tick } from "svelte";

  let { projet, section }: { projet: Projet; section: Section } = $props();
  // Parcours et clarifications ont leur arbre : ils ne sont pas répétés dans les groupes de documents.
  const docsDe = (zone: string) => projet.documents.filter((d) => d.zone === zone && (zone !== "atelier" || !["parcours", "clarification"].includes(d.genre)));

  // La gestion (backlog, plans) vient en tête du volet Projet (REC-017.2).
  const ORDRE: Record<string, [number, string]> = {
    backlog: [-1, "Gestion"],
    plan: [-1, "Gestion"],
    "bdr-analyse": [1, "BDR"],
    "bdr-definition": [1, "BDR"],
    "cahier-des-charges": [2, "Cahier des charges"],
    besoin: [3, "Besoins"],
    exigence: [4, "Exigences"],
    "exigence-nf": [5, "Exigences non fonctionnelles"],
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
  /** Replie tous les accordéons du panneau (REC-008, REC-010). */
  function replier() {
    panneau.querySelectorAll("details").forEach((d) => (d.open = false));
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
    <details open class="arbre">
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
    <button class="replier" onclick={replier} title="Replier tout" aria-label="Replier tout">⊟</button>
  </div>

  {#if section === "produit"}
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
      <details open class="arbre">
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
        <li><a href={url.traitement()} class:actif={actif(url.traitement())}>Traiter les remarques</a></li>
      </ul>
      <details open class="arbre">
        <summary>Pages de l'application <small>{reception.etat.pages.filter(visible).length}</small></summary>
        {@render branche(arbre)}
      </details>
    {/if}
    {#if !groupes.some(([nom]) => nom === "Gestion")}{@render clarifications()}{/if}
    {#each groupes as [nom, g]}
      <details open={g.docs.length <= 12}>
        <summary>{nom} <small>{g.docs.length}</small></summary>
        <ul>
          {#each g.docs as d}
            {@const href = url.document(d.id)}
            <li>
              <a {href} class:actif={actif(href)} title={d.titre}>
                <span class="id">{d.id}</span> {d.titre === d.id ? "" : d.titre}
              </a>
            </li>
          {/each}
        </ul>
      </details>
      {#if nom === "Gestion"}{@render clarifications()}{/if}
    {:else}
      {#if section === "projet"}<p class="discret">Aucun document : <code>wikik nouveau besoin "…"</code>, ou le Design thinking.</p>{/if}
    {/each}
  {/if}
</div>
