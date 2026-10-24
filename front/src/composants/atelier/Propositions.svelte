<script lang="ts">
  // Écran générique « propositions → sélection » (SPEC-20) : le LLM propose, Tibo garde, corrige ou rejette,
  // et peut redemander d'autres propositions. `colonnes` range les cartes par la valeur d'un champ (SCAMPER).
  import type { SousEtape } from "../../types/SousEtape";
  import type { Lot } from "../../types/Lot";
  import { atelier, ecrire } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Valeur from "./Valeur.svelte";
  import Formulaire from "./Formulaire.svelte";

  let {
    etape,
    se,
    cible = null,
    mode,
    colonnes = null,
    entree = null,
  }: { etape: string; se: SousEtape; cible?: string | null; mode: string; colonnes?: { champ: string; valeurs: [string, string][] } | null; entree?: string | null } = $props();

  /** Lot de propositions de la sous-étape (et de la cible si elle est répétée) ; les clarifications ont leurs lots à part. */
  const lot = $derived<Lot | undefined>(
    (etape === "clarification" ? atelier.etat?.lots_clarification : atelier.etat?.etapes.find((e) => e.fiche.id === etape)?.lots)?.find(
      (l) => l.sous_etape === se.id && (l.cible ?? null) === (cible ?? null),
    ),
  );
  /** Demande au LLM en cours. */
  let enCours = $state(false);
  /** Rang de la proposition ouverte en correction. */
  let edition = $state<number | null>(null);
  /** Erreur de la dernière demande ou décision. */
  let erreur = $state("");

  /** Schéma JSON d'une proposition, qui donne les champs du formulaire de correction. */
  const schemaElement = $derived((se.schema as any)?.properties?.elements?.items ?? {});
  /** Propositions qui attendent une décision. */
  const attente = $derived(lot?.elements.filter((e) => e.etat === "en_attente").length ?? 0);
  /** Propositions gardées. */
  const gardes = $derived(lot?.elements.filter((e) => e.etat === "garde").length ?? 0);
  /** Nombre conseillé de propositions à garder dans le mode choisi, s'il y en a un. */
  const conseil = $derived(se.garder ? (mode === "complet" ? se.garder.complet : se.garder.express) : null);

  /** Demande des propositions au LLM (annulable, d'où le drapeau `true` d'`ecrire`) ; les décisions déjà prises sont gardées. */
  async function proposer() {
    enCours = true;
    erreur = "";
    try {
      await ecrire("proposer", { etape, sous_etape: se.id, cible, entree }, true);
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    } finally {
      enCours = false;
    }
  }

  /** Garde, corrige (avec les données modifiées) ou écarte une proposition, désignée par son rang dans le lot. */
  async function decider(indice: number, decision: string, donnees: unknown = null) {
    erreur = "";
    try {
      await ecrire("decider", { etape, sous_etape: se.id, cible, indice, decision, donnees });
      edition = null;
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }

  /** Une ligne pour une proposition décidée : son titre, ou son premier texte. */
  function resume(d: unknown): string {
    const o = (d ?? {}) as Record<string, unknown>;
    for (const k of ["titre", "verbatim", "piste", "analogie", "nom", "de"]) if (typeof o[k] === "string") return k === "de" ? `${o.de} ↔ ${o.vers}` : (o[k] as string);
    const t = Object.values(o).find((x) => typeof x === "string");
    return typeof t === "string" ? t.slice(0, 90) : "proposition";
  }
  /** Propositions avec leur rang dans le lot, qui reste la clé des décisions même après regroupement. */
  const indices = $derived((lot?.elements ?? []).map((p, i) => ({ p, i })));
  /** Cartes rangées par colonne (valeur du champ de `colonnes`, SCAMPER), ou une seule colonne sans titre. */
  const groupes = $derived(
    colonnes
      ? colonnes.valeurs.map(([v, nom]) => ({ nom, items: indices.filter(({ p }) => (p.donnees as any)[colonnes!.champ] === v) }))
      : [{ nom: "", items: indices }],
  );
</script>

<div class="propositions">
  <div class="barre-propositions">
    <button class="principal" onclick={proposer} disabled={enCours}>
      {enCours ? "Le LLM réfléchit…" : lot?.elements.length ? "Redemander" : "Demander des propositions"}
    </button>
    {#if lot?.elements.length}
      <span class="discret">{gardes} gardée(s){conseil ? ` (conseil : ${conseil})` : ""} · {attente} en attente</span>
    {/if}
  </div>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
  {#if lot?.ecartees.length}
    <details class="ecartees"><summary>{lot.ecartees.length} proposition(s) écartée(s) automatiquement</summary>
      <ul>{#each lot.ecartees as e}<li>{e}</li>{/each}</ul>
    </details>
  {/if}
  <div class="colonnes" class:grille={!!colonnes}>
    {#each groupes as g}
      <div class="colonne">
        {#if g.nom}<h4>{g.nom}</h4>{/if}
        {#each g.items as { p, i } (i)}
          <article class="carte-proposition {p.etat}">
            {#if edition === i}
              <Formulaire schema={schemaElement} valeur={p.donnees as Record<string, unknown>} valider={(v) => decider(i, "garder", v)} annuler={() => (edition = null)} />
            {:else if p.etat !== "en_attente"}
              <details class="replie">
                <summary>{resume(p.donnees)}</summary>
                <Valeur v={p.donnees} ordre={Object.keys(schemaElement.properties ?? {})} />
              </details>
              <footer>
                {#if p.etat === "garde"}
                  <span class="pastille statut-validé">gardée</span>
                  {#if p.artefact}<a class="pastille" href={url.document(p.artefact)}>{p.artefact}</a>{/if}
                {:else}
                  <span class="pastille statut-remplacé">rejetée</span>
                {/if}
              </footer>
            {:else}
              <Valeur v={p.donnees} ordre={Object.keys(schemaElement.properties ?? {})} />
              <footer>
                <button class="principal" onclick={() => decider(i, "garder")}>Garder</button>
                <button onclick={() => (edition = i)}>Corriger</button>
                <button class="discret-bouton" onclick={() => decider(i, "rejeter")}>Rejeter</button>
              </footer>
            {/if}
          </article>
        {/each}
      </div>
    {/each}
  </div>
</div>
