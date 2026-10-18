<script lang="ts">
  // Une page de l'application : ce que le code en dit (route, composant, exigences), ses remarques, et la
  // carte pour en ajouter une ; on passe à la page suivante sans revenir à la carte.
  import { reception, remarquesDe, SOURCES } from "../../lib/reception.svelte";
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

  <NouvelleRemarque page={cle} />

  {#each rs as r (r.id)}<CarteRemarque {r} />{/each}
  {#if !rs.length}<p class="discret">Aucune remarque sur cette page pour l'instant.</p>{/if}

  <div class="actions fin-etape">
    {#if precedente}<a class="bouton" href={url.pageReception(precedente.cle)}>← {precedente.nom}</a>{/if}
    {#if suivante}<a class="bouton" href={url.pageReception(suivante.cle)}>{suivante.nom} →</a>{/if}
  </div>
{/if}
