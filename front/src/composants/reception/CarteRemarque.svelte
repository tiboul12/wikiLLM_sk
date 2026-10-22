<script lang="ts">
  // Une remarque : genre, gravité, texte, captures et historique des versions ; la modifier (nouvelle
  // version), la supprimer (toute la chaîne, après confirmation) ; en traitement, statut et explication.
  // Une remarque traitée montre où en est sa correction (commit, poussé, version) et se valide, ou se
  // renvoie à l'agent avec ce qui reste à reprendre (EF-165).
  import type { Remarque } from "../../types/Remarque";
  import EtatGit from "./EtatGit.svelte";
  import { ecrireReception, GENRES, GRAVITES, STATUTS } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";
  import NouvelleRemarque from "./NouvelleRemarque.svelte";

  let { r, traitement = false, avecPage = false }: { r: Remarque; traitement?: boolean; avecPage?: boolean } = $props();
  // svelte-ignore state_referenced_locally
  /** Explication du traitement en une phrase, saisie dans le mode traitement. */
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

  /** Champ « À reprendre » ouvert, pour dire ce qui ne va pas (REC-079, REC-081). */
  let reprise = $state(false);
  /** Ce qui reste à reprendre, renvoyé à l'agent avec la remarque. */
  let commentaire = $state("");
  /** Valide la correction ou la renvoie à reprendre avec le commentaire. */
  async function valider(verdict: "validee" | "a_reprendre") {
    erreur = "";
    try {
      await ecrireReception("valider", { id: r.id, verdict, commentaire });
      reprise = false;
      commentaire = "";
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }
  /** Remarque traitée que l'utilisateur n'a pas encore validée. */
  const aValider = $derived(r.statut === "traitee" && r.validation !== "validee");

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
    {#if r.resolution && !traitement}<p class="resolution"><strong>{STATUTS[r.statut]}</strong> : {r.resolution}</p>{/if}
    {#if r.commit}<p class="git-remarque"><EtatGit commit={r.commit} /></p>{/if}
    {#if r.statut === "traitee"}
      <div class="validation">
        {#if r.validation === "validee"}
          <span class="succes">Validée par toi le {r.valide_le}.</span>
          <button class="discret-bouton" onclick={() => (reprise = !reprise)}>À reprendre…</button>
        {:else}
          <button class="valider-remarque" onclick={() => valider("validee")} title="Le traitement te convient">✓ Je valide</button>
          <button onclick={() => (reprise = !reprise)} title="Rouvrir pour l'agent, avec ce qui reste à faire">↺ À reprendre</button>
        {/if}
        {#if reprise}
          <form class="reprise" onsubmit={(e) => { e.preventDefault(); valider("a_reprendre"); }}>
            <input bind:value={commentaire} placeholder="Ce qui reste à reprendre (l'agent le lira)" aria-label="Ce qui reste à reprendre" />
            <button class="principal" type="submit" disabled={!commentaire.trim()}>Rouvrir</button>
          </form>
        {/if}
      </div>
    {/if}
    {#if traitement}
      <div class="traiter">
        <input bind:value={resolution} placeholder="Ce qui a été fait, ou pourquoi c'est reporté / rejeté" aria-label="Explication" />
        <input class="commit" bind:value={commit} placeholder="commit" aria-label="Commit" />
        <div class="actions">
          {#each ["traitee", "reportee", "rejetee"] as s}
            <button class:principal={s === "traitee"} class:actif={r.statut === s} onclick={() => traiter(s)} disabled={!resolution.trim()}>{STATUTS[s]}</button>
          {/each}
          {#if r.statut !== "ouverte"}<button class="discret-bouton" onclick={() => traiter("ouverte")}>Rouvrir</button>{/if}
        </div>
      </div>
    {/if}
    {#if erreur}<p class="erreur">{erreur}</p>{/if}
  </article>
{/if}
{#if grande}
  <button class="visionneuse" onclick={() => (grande = null)} aria-label="Fermer"><img src={image(grande)} alt="capture agrandie" /></button>
{/if}

<style>
  .validation {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    margin: 8px 0 2px;
  }
  .valider-remarque {
    padding: 4px 12px;
    border: 1px solid var(--vert);
    border-radius: var(--rayon);
    background: var(--vert-fond);
    color: var(--vert);
    font: inherit;
    font-size: 13.5px;
    font-weight: 600;
    cursor: pointer;
  }
  .reprise {
    display: flex;
    gap: 6px;
    flex: 1 1 100%;
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
