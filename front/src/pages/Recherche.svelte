<script lang="ts">
  // Recherche (EF-044, EF-152, EF-172, EF-173) : un champ qui propose au fil de la frappe les documents et les
  // fichiers du projet, par la recherche floue du graphe (REC-087) ; Entrée lance la recherche plein texte dans
  // les blocs de doc et les documents ; « Par le sens » trouve ce qui répond à une question sans les mots exacts
  // (embeddings d'Ollama, ADR-0020, REC-097) ; « Demander à l'IA » : le LLM configuré lit le projet et rend les
  // éléments qui répondent, liés, chacun avec sa raison (ni l'un ni l'autre dans le site exporté).
  import { api } from "../lib/api";
  import { url, aller } from "../lib/routeur.svelte";
  import type { Resultat } from "../types/Resultat";
  import { atelier } from "../lib/atelier.svelte";
  import { autoLien } from "../lib/connus.svelte";
  import { rendre } from "../lib/markdown";
  import { statique } from "../lib/statique";
  import type { Projet } from "../types/Projet";
  import type { Trouve } from "../types/Trouve";
  import ChampFloue from "../composants/ChampFloue.svelte";
  import type { Cherchable } from "../lib/floue";

  let { q, projet }: { q: string; projet: Projet | null } = $props();
  /** Ce que le champ propose au fil de la frappe : les documents et les fichiers du projet. */
  const cherchables = $derived<Cherchable[]>([
    ...(projet?.documents ?? []).map((d) => ({ id: d.id, libelle: d.titre, detail: d.genre })),
    ...(projet?.fichiers ?? []).map((f) => ({ id: f.chemin, libelle: f.chemin.split("/").pop() ?? f.chemin, detail: "fichier" })),
  ]);
  /** Choix dans le champ : un élément proposé s'ouvre, un texte lance la recherche plein texte. */
  function choisir(c: Cherchable | string) {
    if (typeof c !== "string") return aller(autoLien(c.id) ?? url.document(c.id));
    if (c) aller(url.recherche(c));
  }

  /** Résultats de la recherche par le sens ; `null` tant qu'elle n'a pas été lancée. */
  let sens = $state<Trouve[] | null>(null);
  /** Erreur de la recherche par le sens (Ollama absent, modèle à installer). */
  let sensErreur = $state("");
  /** Recherche par le sens en cours (la première calcule tous les vecteurs). */
  let sensEnCours = $state(false);
  /** Lance la recherche par le sens sur le texte du champ. */
  async function parLeSens() {
    const question = saisie.trim();
    if (!question) return;
    sens = null;
    sensErreur = "";
    sensEnCours = true;
    try {
      const r = await fetch("/api/recherche/sens", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ question }) });
      const v = await r.json().catch(() => ({}));
      if (!r.ok) throw new Error(v.erreur ?? `${r.status}`);
      sens = v as Trouve[];
    } catch (x) {
      sensErreur = x instanceof Error ? x.message : String(x);
    } finally {
      sensEnCours = false;
    }
  }
  /** Lien d'un résultat par le sens : symbole dans sa page de fichier, ou document. */
  const lienSens = (t: Trouve) => autoLien(t.cle) ?? url.document(t.cle);
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
  <ChampFloue elements={cherchables} bind:valeur={saisie} {choisir} placeholder="Un document, un fichier (ef-140, onglets…), des mots ou une question" libelle="Rechercher" />
  <button type="submit" disabled={!saisie.trim()}>Chercher</button>
  {#if !statique}
    <button type="button" onclick={parLeSens} disabled={!saisie.trim() || sensEnCours} title="Ce qui répond à la question, même sans les mots exacts (embeddings d'Ollama)">≈ Par le sens</button>
    <button type="button" class="principal" onclick={demanderIa} disabled={!saisie.trim() || iaEnCours} title="Le LLM lit le projet et répond avec les éléments trouvés">✦ Demander à l'IA</button>
  {/if}
</form>

{#if sensEnCours}
  <p class="discret">Recherche par le sens… (la première fois, wikik calcule les vecteurs de tout le projet)</p>
{:else if sensErreur}
  <p class="erreur">{sensErreur}</p>
{:else if sens}
  <section class="reponse-ia">
    <h2>Par le sens</h2>
    {#if sens.length === 0}<p>Aucun résultat.</p>{/if}
    <ul class="resultats">
      {#each sens as t (t.cle)}
        <li><span class="genre">{t.genre}</span> <a href={lienSens(t)}>{t.genre === "symbole" ? t.cle.split("::").pop() : `${t.cle} · ${t.titre}`}</a> <span class="discret">{t.genre === "symbole" ? t.cle.split("::")[0] : ""} · {Math.round(t.score * 100)} %</span></li>
      {/each}
    </ul>
  </section>
{/if}

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
