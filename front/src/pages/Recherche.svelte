<script lang="ts">
  // Recherche (EF-044, EF-152) : un champ, la recherche plein texte dans les blocs de doc et les documents,
  // et « Demander à l'IA » : le LLM configuré lit le projet et rend les éléments qui répondent, liés, chacun
  // avec sa raison (pas dans le site exporté).
  import { api } from "../lib/api";
  import { url, aller } from "../lib/routeur.svelte";
  import type { Resultat } from "../types/Resultat";
  import { atelier } from "../lib/atelier.svelte";
  import { autoLien } from "../lib/connus.svelte";
  import { rendre } from "../lib/markdown";
  import { statique } from "../lib/statique";

  let { q }: { q: string } = $props();
  // svelte-ignore state_referenced_locally
  /** Texte du champ, initialisé avec la recherche de l'adresse. */
  let saisie = $state(q);
  /** Résultats plein texte ; `null` tant qu'aucune recherche n'a été faite. */
  let resultats = $state<Resultat[] | null>(null);
  // svelte-ignore state_referenced_locally
  if (q) api.cherche(q).then((r) => (resultats = r)).catch(() => (resultats = []));

  /** Élément rendu par le LLM : identifiant, chemin ou symbole, et pourquoi il répond. */
  type ReponseIa = { cible: string; raison: string };
  /** Réponse du LLM ; `null` tant qu'il n'a pas été interrogé. */
  let ia = $state<ReponseIa[] | null>(null);
  /** Erreur de l'appel au LLM (non configuré, annulé, réponse illisible). */
  let iaErreur = $state("");
  /** Appel au LLM en cours. */
  let iaEnCours = $state(false);

  /** Lien d'un résultat : le symbole dans sa page de fichier, ou le document. */
  const lien = (r: Resultat) => (r.fichier ? url.fichier(r.fichier, r.titre) : url.document(r.cle));

  /** Relance la recherche via l'adresse, pour qu'elle reste dans l'historique et se partage. */
  function chercher(e: SubmitEvent) {
    e.preventDefault();
    if (saisie.trim()) aller(url.recherche(saisie.trim()));
  }

  /**
   * Pose la question au LLM du projet. L'appel compte dans `atelier.occupe` pour afficher le bandeau d'annulation,
   * comme les appels de l'atelier.
   */
  async function demanderIa() {
    const question = saisie.trim();
    if (!question) return;
    ia = null;
    iaErreur = "";
    iaEnCours = true;
    if (!atelier.occupe) atelier.depuis = Date.now();
    atelier.occupe++;
    try {
      const r = await fetch("/api/recherche/ia", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ question }) });
      const v = await r.json().catch(() => ({}));
      if (!r.ok) throw new Error(v.erreur ?? `${r.status}`);
      ia = v as ReponseIa[];
    } catch (x) {
      iaErreur = x instanceof Error ? x.message : String(x);
    } finally {
      iaEnCours = false;
      atelier.occupe--;
    }
  }
</script>

<h1>Recherche</h1>
<form class="champ-recherche" onsubmit={chercher} role="search">
  <input type="search" bind:value={saisie} placeholder="Mots à chercher, ou une question pour l'IA…" aria-label="Rechercher" />
  <button type="submit" disabled={!saisie.trim()}>Chercher</button>
  {#if !statique}
    <button type="button" class="principal" onclick={demanderIa} disabled={!saisie.trim() || iaEnCours} title="Le LLM lit le projet et répond avec les éléments trouvés">✦ Demander à l'IA</button>
  {/if}
</form>

{#if iaEnCours}
  <p class="discret">L'IA lit le projet…</p>
{:else if iaErreur}
  <p class="erreur">{iaErreur}</p>
{:else if ia}
  <section class="reponse-ia">
    <h2>Réponse de l'IA</h2>
    {#if ia.length === 0}<p>Rien dans le projet ne répond à cette question.</p>{/if}
    <ul class="resultats">
      {#each ia as r (r.cible)}
        {@const href = autoLien(r.cible)}
        <li>
          {#if href}<a {href}><code>{r.cible}</code></a>{:else}<code>{r.cible}</code>{/if}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="raison">{@html rendre(r.raison, autoLien, undefined, undefined, autoLien)}</div>
        </li>
      {/each}
    </ul>
  </section>
{/if}

{#if q}
  <h2>Résultats pour « {q} »</h2>
  {#if resultats === null}
    <p class="discret">Recherche…</p>
  {:else if resultats.length === 0}
    <p>Aucun résultat.</p>
  {:else}
    <ul class="resultats">
      {#each resultats as r}
        <li>
          <span class="genre">{r.genre}</span>
          <a href={lien(r)}>{r.genre === "symbole" ? r.titre : `${r.cle} · ${r.titre}`}</a>
          {#if r.fichier}<span class="discret">{r.fichier}</span>{/if}
          <p>{r.extrait}</p>
        </li>
      {/each}
    </ul>
  {/if}
{/if}

<style>
  .champ-recherche {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 1rem;
  }
  .champ-recherche input {
    flex: 1 1 18rem;
    min-width: 0;
  }
  .reponse-ia {
    border-left: 3px solid var(--accent, #4c8dff);
    padding-left: 1rem;
    margin-bottom: 1.5rem;
  }
  .raison :global(p) {
    margin: 0.2rem 0 0;
  }
</style>
