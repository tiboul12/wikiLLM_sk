<script lang="ts">
  // Une remarque : genre, gravité, texte, captures et historique des versions ; la modifier (nouvelle
  // version), la supprimer (toute la chaîne, après confirmation). Même interface sur la carte d'une page et
  // dans le Traitement (EF-181, REC-085) : un curseur d'états ouverte → traitée → validée (REC-086) dit où en
  // est la remarque et la fait avancer (traitée demande l'explication et le commit ; validée est le geste de
  // l'utilisateur ; revenir à « ouverte » demande ce qui reste à reprendre), reportée et rejetée à côté. Une
  // remarque traitée montre où en est sa correction (commit, poussé, version : EF-165).
  import type { Remarque } from "../../types/Remarque";
  import EtatGit from "./EtatGit.svelte";
  import { ecrireReception, GENRES, GRAVITES, STATUTS } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";
  import NouvelleRemarque from "./NouvelleRemarque.svelte";

  let { r, avecPage = false }: { r: Remarque; avecPage?: boolean } = $props();
  // svelte-ignore state_referenced_locally
  /** Explication du traitement en une phrase, saisie quand on passe la remarque à traitée, reportée ou rejetée. */
  let resolution = $state(r.resolution ?? "");
  // svelte-ignore state_referenced_locally
  /** Commit de la correction, facultatif. */
  let commit = $state(r.commit ?? "");
  /** Capture affichée en grand, ou aucune. */
  let grande = $state<string | null>(null);
  /** Erreur de la dernière action sur la remarque. */
  let erreur = $state("");
  /** Formulaire de modification ouvert (une modification crée une nouvelle version). */
  let edition = $state(false);
  /** Demande de confirmation de la suppression affichée. */
  let confirmer = $state(false);

  /** Enregistre le statut, l'explication et le commit de la remarque. */
  async function traiter(statut: string) {
    erreur = "";
    try {
      await ecrireReception("traiter", { id: r.id, statut, resolution, commit: commit || null });
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }

  /** Ce qui reste à reprendre, renvoyé à l'agent avec la remarque. */
  let commentaire = $state("");
  /** Valide la correction ou la renvoie à reprendre avec le commentaire.
   * @relu 2026-10-24 452cff86
   */
  async function valider(verdict: "validee" | "a_reprendre") {
    erreur = "";
    try {
      await ecrireReception("valider", { id: r.id, verdict, commentaire });
      commentaire = "";
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }
  /** Remarque traitée que l'utilisateur n'a pas encore validée. */
  const aValider = $derived(r.statut === "traitee" && r.validation !== "validee");

  /** Étapes du curseur : qui fait avancer la remarque à chacune. */
  const ETAPES: [string, string, string][] = [
    ["ouverte", "Ouverte", "déposée par toi"],
    ["traitee", "Traitée", "par l'agent"],
    ["validee", "Validée", "par toi"],
  ];
  /** Rang de la remarque sur le curseur ; -1 si elle est reportée ou rejetée (hors du curseur). */
  const rangEtat = $derived(r.validation === "validee" ? 2 : r.statut === "traitee" ? 1 : r.statut === "ouverte" ? 0 : -1);
  /** Formulaire ouvert par un geste du curseur : explication (traitée, reportée, rejetée) ou reprise. */
  let cible = $state<"traitee" | "reportee" | "rejetee" | "reprise" | null>(null);
  /** Geste sur une étape du curseur : valider va d'un clic, le reste ouvre le formulaire qui convient. */
  function allerA(i: number) {
    erreur = "";
    if (i === rangEtat) return;
    if (i === 2) {
      if (r.statut === "traitee") valider("validee");
      else erreur = "Une remarque se valide une fois traitée.";
    } else if (i === 1) cible = "traitee";
    else if (rangEtat > 0) cible = "reprise";
    else traiter("ouverte");
  }
  /** Enregistre l'explication du formulaire avec le statut visé. */
  async function confirmerTraitement() {
    if (cible && cible !== "reprise") await traiter(cible);
    if (!erreur) cible = null;
  }

  /** Supprime la remarque et toutes ses versions ; en cas d'échec, la confirmation se referme. */
  async function supprimer() {
    erreur = "";
    try {
      await ecrireReception("supprimer", { id: r.id });
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
      confirmer = false;
    }
  }
  /** Adresse d'une capture servie par le serveur. */
  const image = (chemin: string) => `/api/image?chemin=${encodeURIComponent(chemin)}`;
  /** Versions de la remarque, de la première à la courante ; la remarque seule si elle n'a jamais été modifiée. */
  const versions = $derived(r.historique.length ? r.historique : [r.id]);
</script>

{#if edition}
  <NouvelleRemarque page={r.page} remarque={r} fini={() => (edition = false)} />
{:else}
  <article class="remarque gravite-{r.gravite} statut-{r.statut}">
    <header>
      <a class="id" href={url.document(r.id)}>{r.id}</a>
      <span class="pastille genre-{r.genre}">{GENRES[r.genre] ?? r.genre}</span>
      <span class="pastille gravite">{GRAVITES[r.gravite] ?? r.gravite}</span>
      {#if r.statut !== "ouverte"}<span class="pastille statut">{STATUTS[r.statut] ?? r.statut}</span>{/if}
      {#if r.validation === "validee"}<span class="pastille validee" title="Validée le {r.valide_le}">✓ validée</span>{:else if aValider}<span class="pastille a-valider">à valider</span>{/if}
      {#if avecPage}<a class="page-de" href={url.pageReception(r.page)}>{r.nom_page}</a>{/if}
      <span class="outils-remarque">
        <span class="discret date">{r.date}</span>
        <button class="discret-bouton" onclick={() => (edition = true)} title="Modifier : crée une nouvelle version">✎ Modifier</button>
        <button class="discret-bouton danger" onclick={() => (confirmer = true)} title="Supprimer la remarque et son historique">✕ Supprimer</button>
      </span>
    </header>
    {#if r.texte}<p class="texte-libre">{r.texte}</p>{/if}
    {#if r.images.length}
      <div class="captures">
        {#each r.images as i}
          <button class="capture" onclick={() => (grande = i)} title="Agrandir"><img src={image(i)} alt="capture" loading="lazy" /></button>
        {/each}
      </div>
    {/if}
    {#if r.historique.length > 1}
      <p class="historique">
        <span class="discret">Historique :</span>
        {#each r.historique as v, n}{#if n}<span class="discret">{" → "}</span>{/if}<a class:courante={v === r.id} href={url.document(v)}>{v}</a>{/each}
      </p>
    {/if}
    {#if confirmer}
      <div class="confirmation" role="alertdialog">
        Supprimer {versions.join(", ")} et {versions.length > 1 ? "leurs" : "ses"} captures ? (Le dépôt git en garde la trace.)
        <div class="actions">
          <button class="principal danger" onclick={supprimer}>Supprimer</button>
          <button onclick={() => (confirmer = false)}>Annuler</button>
        </div>
      </div>
    {/if}
    {#if r.resolution}<p class="resolution"><strong>{STATUTS[r.statut]}</strong> : {r.resolution}</p>{/if}
    {#if r.commit}<p class="git-remarque"><EtatGit commit={r.commit} /></p>{/if}
    <div class="parcours-remarque">
      <div class="curseur" role="group" aria-label="État de la remarque" style="--rang: {Math.max(rangEtat, 0)}" class:hors={rangEtat < 0}>
        <span class="piste"><span class="rempli"></span></span>
        {#each ETAPES as [cle, nom, qui], i (cle)}
          <button class="etape-etat" class:faite={rangEtat >= i} class:courante={rangEtat === i} onclick={() => allerA(i)} title={i === rangEtat ? nom : `Passer à « ${nom} »`}>
            <span class="plot"></span><span class="nom-etat">{nom}</span><small>{qui}</small>
          </button>
        {/each}
      </div>
      <span class="a-cote">
        {#if rangEtat < 0}<span class="pastille statut">{STATUTS[r.statut]}</span>{/if}
        <button class="discret-bouton" class:actif={r.statut === "reportee"} onclick={() => (cible = "reportee")}>Reporter</button>
        <button class="discret-bouton" class:actif={r.statut === "rejetee"} onclick={() => (cible = "rejetee")}>Rejeter</button>
      </span>
    </div>
    {#if cible === "reprise"}
      <form class="reprise" onsubmit={(e) => { e.preventDefault(); valider("a_reprendre").then(() => (cible = null)); }}>
        <input bind:value={commentaire} placeholder="Ce qui reste à reprendre (l'agent le lira)" aria-label="Ce qui reste à reprendre" />
        <button class="principal" type="submit" disabled={!commentaire.trim()}>Rouvrir</button>
        <button type="button" class="discret-bouton" onclick={() => (cible = null)}>Annuler</button>
      </form>
    {:else if cible}
      <form class="traiter" onsubmit={(e) => { e.preventDefault(); confirmerTraitement(); }}>
        <input bind:value={resolution} placeholder="Ce qui a été fait, ou pourquoi c'est reporté / rejeté (une phrase)" aria-label="Explication" />
        <input class="commit" bind:value={commit} placeholder="commit" aria-label="Commit" />
        <div class="actions">
          <button class="principal" type="submit" disabled={!resolution.trim()}>{STATUTS[cible]}</button>
          <button type="button" class="discret-bouton" onclick={() => (cible = null)}>Annuler</button>
        </div>
      </form>
    {/if}
    {#if erreur}<p class="erreur">{erreur}</p>{/if}
  </article>
{/if}
{#if grande}
  <button class="visionneuse" onclick={() => (grande = null)} aria-label="Fermer"><img src={image(grande)} alt="capture agrandie" /></button>
{/if}

<style>
  .parcours-remarque {
    display: flex;
    align-items: center;
    gap: 12px 18px;
    flex-wrap: wrap;
    margin: 10px 0 4px;
  }
  /* Curseur d'états (REC-086) : trois plots sur une piste, remplie jusqu'à l'état courant. */
  .curseur {
    position: relative;
    display: grid;
    grid-template-columns: repeat(3, minmax(84px, 1fr));
    flex: 1 1 280px;
    max-width: 460px;
  }
  .piste {
    position: absolute;
    left: calc(100% / 6);
    right: calc(100% / 6);
    top: 8px;
    height: 3px;
    border-radius: 2px;
    background: var(--fond-3);
  }
  .rempli {
    display: block;
    height: 100%;
    width: calc(var(--rang) * 50%);
    border-radius: 2px;
    background: var(--vert);
    transition: width 0.2s;
  }
  .curseur.hors .rempli {
    width: 0;
  }
  .etape-etat {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    border: 0;
    background: none;
    padding: 0;
    color: var(--texte-2);
    font: inherit;
    cursor: pointer;
  }
  .plot {
    width: 19px;
    height: 19px;
    border-radius: 50%;
    border: 2px solid var(--bord);
    background: var(--fond);
    transition: background 0.2s, border-color 0.2s;
  }
  .faite .plot {
    border-color: var(--vert);
    background: var(--vert-fond);
  }
  .courante .plot {
    background: var(--vert);
    box-shadow: 0 0 0 3px var(--vert-fond);
  }
  .nom-etat {
    font-size: 13px;
    font-weight: 600;
  }
  .courante .nom-etat {
    color: var(--texte);
  }
  .etape-etat small {
    font-size: 11px;
  }
  .etape-etat:hover .plot {
    border-color: var(--accent);
  }
  .a-cote {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .a-cote .actif {
    color: var(--texte);
    font-weight: 600;
  }
  .reprise,
  .traiter {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin: 6px 0;
  }
  .reprise input,
  .traiter input:not(.commit) {
    flex: 1 1 260px;
  }
  .traiter .commit {
    width: 8rem;
  }
  .reprise input {
    flex: 1;
  }
  .pastille.validee {
    background: var(--vert-fond);
    color: var(--vert);
  }
  .pastille.a-valider {
    background: var(--orange-fond);
    color: var(--orange);
  }
  .git-remarque {
    margin: 6px 0 0;
  }
</style>
