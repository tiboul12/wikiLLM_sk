<script lang="ts">
  // Une page de l'application : le lien pour l'ouvrir dans l'application, ce que le code en dit (route,
  // composant, exigences), ses remarques, et la carte pour en ajouter une ; on passe à la page suivante
  // sans revenir à la carte.
  import { reception, remarquesDe, lienApplication, SOURCES } from "../../lib/reception.svelte";
  import { url } from "../../lib/routeur.svelte";
  import Code from "../../composants/reception/Code.svelte";
  import CarteRemarque from "../../composants/reception/CarteRemarque.svelte";
  import NouvelleRemarque from "../../composants/reception/NouvelleRemarque.svelte";

  let { cle }: { cle: string } = $props();
  const visibles = $derived(reception.etat?.pages.filter((p) => !p.masque) ?? []);
  const i = $derived(visibles.findIndex((p) => p.cle === cle));
  const p = $derived(reception.etat?.pages.find((x) => x.cle === cle));
  const rs = $derived(remarquesDe(cle));
  const precedente = $derived(i > 0 ? visibles[i - 1] : null);
  const suivante = $derived(i >= 0 && i < visibles.length - 1 ? visibles[i + 1] : null);
</script>

{#if !p}
  <p class="discret">{reception.etat ? `Page inconnue : ${cle}` : "Chargement…"}</p>
{:else}
  <p class="fil"><a href={url.reception()}>Réception</a>{i >= 0 ? ` · page ${i + 1} / ${visibles.length}` : ""}</p>
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
      <a class="bouton" href={lien} target="_blank" rel="noopener">Ouvrir cette page dans l'application ↗</a>
      <span class="discret">{lien}</span>
    {:else if p.cle === "general"}
      <span class="discret">Remarques qui ne visent pas un écran en particulier.</span>
    {:else if !reception.etat?.url_application}
      <span class="discret">Pour un lien direct vers cette page : saisis l'adresse de l'application en tête de la <a href={url.reception()}>carte</a>.</span>
    {:else if p.source === "enum"}
      <span class="discret">Écran affiché par état (pas d'adresse directe) : ouvre l'application et va sur « {p.nom} ».</span>
    {:else}
      <span class="discret">Route inconnue : ajoute-la avec ✎ sur la <a href={url.reception()}>carte</a> pour avoir le lien direct.</span>
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
