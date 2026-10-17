<script lang="ts">
  // Free writing : pour chaque axe, écrire sans s'arrêter pendant le temps imparti, puis récolter les
  // unités de sens (verbatims extraits mot pour mot par le LLM).
  import type { Fiche } from "../../types/Fiche";
  import { api } from "../../lib/api";
  import { deType, ecrire } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Propositions from "./Propositions.svelte";

  let { fiche, mode }: { fiche: Fiche; mode: string } = $props();
  const ecriture = $derived(fiche.sous_etapes.find((s) => s.id === "ecriture")!);
  const recolte = $derived(fiche.sous_etapes.find((s) => s.id === "recolte")!);
  const minutes = $derived(ecriture.duree_minutes ? (mode === "complet" ? ecriture.duree_minutes.complet : ecriture.duree_minutes.express) : 10);
  const axes = $derived(deType("axe"));
  const ecrits = $derived(deType("ecrit"));

  let axeOuvert = $state<string | null>(null);
  let texte = $state("");
  let fin = $state(0);
  let maintenant = $state(Date.now());
  let erreur = $state("");
  let textes = $state<Record<string, string>>({});

  $effect(() => {
    if (!fin) return;
    const t = setInterval(() => (maintenant = Date.now()), 1000);
    return () => clearInterval(t);
  });
  const reste = $derived(Math.max(0, Math.round((fin - maintenant) / 1000)));

  function commencer(axe: string) {
    axeOuvert = axe;
    texte = "";
    fin = Date.now() + minutes * 60_000;
    maintenant = Date.now();
  }

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
  const mmss = (s: number) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
</script>

{#if !axes.length}
  <p class="discret">Garde d'abord un ou plusieurs axes (sous-étape précédente).</p>
{/if}
{#each axes as axe (axe.id)}
  <section class="cible">
    <h4><a class="id" href={url.document(axe.id)}>{axe.id}</a> {axe.titre}</h4>
    {#if axeOuvert === axe.id}
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
