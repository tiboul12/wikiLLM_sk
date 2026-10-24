<script lang="ts">
  // Carte de réception d'une page de l'application (EF-140) : le lien qui l'ouvre dans un nouvel onglet (adresse saisie ou déduite du
  // projet ; saisie sur place si rien n'est déduit), ce que le code en dit (route,
  // composant, exigences), ses remarques, et la carte pour en ajouter une ; on passe à la page suivante
  // sans revenir à la carte.
  import { reception, remarquesDe, lienApplication, ecrireReception, visible, SOURCES } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Code from "../../composants/reception/Code.svelte";
  import CarteRemarque from "../../composants/reception/CarteRemarque.svelte";
  import NouvelleRemarque from "../../composants/reception/NouvelleRemarque.svelte";

  let { cle }: { cle: string } = $props();
  const visibles = $derived(reception.etat?.pages.filter(visible) ?? []);
  const i = $derived(visibles.findIndex((p) => p.cle === cle));
  const p = $derived(reception.etat?.pages.find((x) => x.cle === cle));
  const rs = $derived(remarquesDe(cle));
  const precedente = $derived(i > 0 ? visibles[i - 1] : null);
  let adresse = $state("");
  async function enregistrerAdresse(e: SubmitEvent) {
    e.preventDefault();
    if (adresse.trim()) await ecrireReception("application", { url: adresse.trim() }).catch(() => {});
  }
  const suivante = $derived(i >= 0 && i < visibles.length - 1 ? visibles[i + 1] : null);
</script>

{#if !p}
  <p class="discret">{reception.etat ? `Page inconnue : ${cle}` : "Chargement…"}</p>
{:else}
  <p class="fil"><a href={url.reception()}>Cahier de réception</a> · carte de réception{i >= 0 ? ` ${i + 1} / ${visibles.length}` : ""}</p>
  <h1>{p.nom}</h1>
  <div class="meta">
    {#if p.route}<code>{p.route}</code>{/if}
    {#if SOURCES[p.source]}<span class="discret">{SOURCES[p.source]}</span>{/if}
    {#each p.symboles as s}<Code id={s} />{/each}
    {#each p.exigences as ex}<a class="pastille" href={url.document(ex)}>{ex}</a>{/each}
  </div>

  {@const lien = lienApplication(p.route)}
  <div class="lien-application">
    {#if lien}
      <a class="bouton principal" href={lien} target="_blank" rel="noopener">{p.route ? "Ouvrir cette page" : "Ouvrir l'application"} ↗</a>
      <a class="discret" href={lien} target="_blank" rel="noopener">{lien}</a>
      {#if !p.route && p.cle !== "general"}<span class="discret">(pas d'adresse directe pour cet écran : ✎ sur la carte pour en donner une)</span>{/if}
    {:else}
      <form class="adresse-application" onsubmit={enregistrerAdresse}>
        <label for="adresse-page">Adresse de l'application</label>
        <input id="adresse-page" bind:value={adresse} placeholder="http://10.0.0.21:4200" />
        <button type="submit">Enregistrer</button>
      </form>
    {/if}
  </div>

  <NouvelleRemarque page={cle} />

  {#each rs as r (r.id)}<CarteRemarque {r} />{/each}
  {#if !rs.length}<p class="discret">Aucune remarque sur cette page pour l'instant.</p>{/if}

  <div class="actions fin-etape">
    {#if precedente}<a class="bouton" href={url.pageReception(precedente.cle)}>← {precedente.nom}</a>{/if}
    {#if suivante}<a class="bouton" href={url.pageReception(suivante.cle)}>{suivante.nom} →</a>{/if}
  </div>
{/if}
