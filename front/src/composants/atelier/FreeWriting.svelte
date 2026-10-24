<script lang="ts">
  // Free writing : pour chaque axe, écrire sans s'arrêter pendant le temps imparti, puis récolter les
  // unités de sens (verbatims extraits mot pour mot par le LLM). La proposition de l'axe (ce que l'écriture
  // doit explorer) reste affichée juste au-dessus de l'écriture (REC-096) ; un axe se met en pause et se
  // reprend (REC-095, EF-179) : en pause, il n'est plus proposé à l'écriture.
  import type { Fiche } from "../../types/Fiche";
  import { api } from "../../lib/api";
  import { atelier, deType, ecrire } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Propositions from "./Propositions.svelte";

  let { fiche, mode }: { fiche: Fiche; mode: string } = $props();
  /** Sous-étape d'écriture de la fiche (durée Express et Complet). */
  const ecriture = $derived(fiche.sous_etapes.find((s) => s.id === "ecriture")!);
  /** Sous-étape de récolte des unités de sens, rendue sous chaque écrit. */
  const recolte = $derived(fiche.sous_etapes.find((s) => s.id === "recolte")!);
  /** Durée d'écriture du mode choisi, 10 minutes si la fiche n'en donne pas. */
  const minutes = $derived(ecriture.duree_minutes ? (mode === "complet" ? ecriture.duree_minutes.complet : ecriture.duree_minutes.express) : 10);
  /** Axes du pitch : un écrit libre par axe. */
  const axes = $derived(deType("axe"));
  /** Écrits libres déjà enregistrés. */
  const ecrits = $derived(deType("ecrit"));

  /** Axe dont l'écriture chronométrée est en cours. */
  let axeOuvert = $state<string | null>(null);
  /** Texte en cours d'écriture, envoyé tel quel (pas de correction : principe de non-altération). */
  let texte = $state("");
  /** Heure de fin du chrono, en millisecondes ; 0 hors écriture. */
  let fin = $state(0);
  /** Heure courante, rafraîchie chaque seconde pendant l'écriture pour le décompte. */
  let maintenant = $state(Date.now());
  /** Erreur d'enregistrement de l'écrit. */
  let erreur = $state("");
  /** Texte de chaque écrit enregistré, par identifiant, chargé une fois pour la récolte. */
  let textes = $state<Record<string, string>>({});
  /** Proposition de chaque axe (le corps de son fichier, sans le titre), chargée une fois. */
  let propositions = $state<Record<string, string>>({});
  $effect(() => {
    for (const a of axes)
      if (!(a.id in propositions)) api.document(a.id).then((p) => (propositions[a.id] = p.corps.replace(/^#[^\n]*\n/, "").trim()));
  });
  /** Axes en pause dans cet atelier. */
  const pauses = $derived(new Set(atelier.etat?.pauses ?? []));
  /** Met l'axe en pause ou le reprend ; une écriture en cours sur lui est abandonnée. */
  async function basculerPause(axe: string) {
    erreur = "";
    try {
      if (axeOuvert === axe) axeOuvert = null;
      await ecrire("pause", { axe, pause: !pauses.has(axe) });
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }

  $effect(() => {
    if (!fin) return;
    const t = setInterval(() => (maintenant = Date.now()), 1000);
    return () => clearInterval(t);
  });
  /** Secondes restantes ; le chrono n'arrête pas l'écriture, il l'indique. */
  const reste = $derived(Math.max(0, Math.round((fin - maintenant) / 1000)));

  /** Ouvre l'écriture sur un axe et lance le chrono de la durée du mode. */
  function commencer(axe: string) {
    axeOuvert = axe;
    texte = "";
    fin = Date.now() + minutes * 60_000;
    maintenant = Date.now();
  }

  /** Enregistre l'écrit de l'axe avec la durée réellement passée (une minute au moins) et ferme l'écriture. */
  async function terminer() {
    erreur = "";
    try {
      const duree = Math.max(1, Math.round((minutes * 60 - reste) / 60));
      await ecrire("saisir", { etape: "free-writing", sous_etape: "ecriture", cible: axeOuvert, donnees: { texte, duree_minutes: duree } });
      axeOuvert = null;
      fin = 0;
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }

  $effect(() => {
    for (const e of ecrits)
      if (!(e.id in textes)) api.document(e.id).then((p) => (textes[e.id] = p.corps.trim()));
  });
  /** Durée en minutes:secondes pour le chrono. */
  const mmss = (s: number) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
</script>

{#if !axes.length}
  <p class="discret">Garde d'abord un ou plusieurs axes (sous-étape précédente).</p>
{/if}
{#each axes as axe (axe.id)}
  {@const enPause = pauses.has(axe.id)}
  <section class="cible" class:en-pause={enPause}>
    <h4>
      <a class="id" href={url.document(axe.id)}>{axe.id}</a> {axe.titre}
      {#if enPause}<span class="pastille">en pause</span>{/if}
      <button class="discret-bouton pause" onclick={() => basculerPause(axe.id)}>{enPause ? "▶ Reprendre" : "⏸ Mettre en pause"}</button>
    </h4>
    {#if propositions[axe.id]}<p class="proposition-axe">{propositions[axe.id]}</p>{/if}
    {#if enPause}
      <p class="discret">Axe en pause : il n'est pas proposé à l'écriture.</p>
    {:else if axeOuvert === axe.id}
      <div class="ecriture">
        <div class="minuteur" class:fini={reste === 0}>{reste ? mmss(reste) : "Temps écoulé : termine ta phrase"}</div>
        <!-- svelte-ignore a11y_autofocus -->
        <textarea autofocus rows={14} bind:value={texte} placeholder="Écris sans t'arrêter, sans te relire, sans te censurer."></textarea>
        <div class="actions">
          <button class="principal" onclick={terminer} disabled={!texte.trim()}>Terminer ({texte.split(/\s+/).filter(Boolean).length} mots)</button>
          <button onclick={() => (axeOuvert = null)}>Abandonner</button>
        </div>
      </div>
    {:else}
      <button onclick={() => commencer(axe.id)}>Écrire sur cet axe ({minutes} min)</button>
    {/if}
    {#each ecrits.filter((e) => e.frontmatter.axe === axe.id) as e (e.id)}
      <details class="ecrit" open>
        <summary><a class="id" href={url.document(e.id)}>{e.id}</a> texte libre</summary>
        <blockquote class="texte-libre">{textes[e.id] ?? "…"}</blockquote>
        <h5>Unités de sens</h5>
        <Propositions etape="free-writing" se={recolte} cible={e.id} {mode} />
      </details>
    {/each}
  </section>
{/each}
{#if erreur}<p class="erreur">{erreur}</p>{/if}

<style>
  .proposition-axe {
    margin: 2px 0 10px;
    padding: 8px 12px;
    border-left: 3px solid var(--accent);
    background: var(--accent-fond);
    border-radius: 0 var(--rayon) var(--rayon) 0;
    font-size: 14.5px;
  }
  .pause {
    margin-left: 8px;
    font-size: 12.5px;
  }
  .en-pause {
    opacity: 0.65;
  }
</style>
