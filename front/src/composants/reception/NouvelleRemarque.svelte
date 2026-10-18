<script lang="ts">
  // Saisie d'une remarque : texte, genre, gravité, captures. Une carte = une remarque ; Ctrl+Entrée
  // enregistre. Avec `remarque`, la saisie en écrit la version suivante (SPEC-23 R8).
  import type { Remarque } from "../../types/Remarque";
  import { ecrireReception, GENRES, GRAVITES } from "../../lib/reception.svelte";
  import Captures from "./Captures.svelte";

  let { page, remarque = null, fini }: { page: string; remarque?: Remarque | null; fini?: (id: string) => void } = $props();
  // svelte-ignore state_referenced_locally
  let texte = $state(remarque?.texte ?? "");
  // svelte-ignore state_referenced_locally
  let genre = $state(remarque?.genre ?? "ergonomie");
  // svelte-ignore state_referenced_locally
  let gravite = $state(remarque?.gravite ?? "genant");
  let images = $state<string[]>([]);
  // svelte-ignore state_referenced_locally
  let gardees = $state<string[]>(remarque ? [...remarque.images] : []);
  let erreur = $state("");
  let enCours = $state(false);
  let message = $state("");
  let formulaire = $state<HTMLFormElement>();
  const vide = $derived(!texte.trim() && !images.length && !gardees.length);

  async function enregistrer(e?: Event) {
    e?.preventDefault();
    if (vide) return;
    enCours = true;
    erreur = message = "";
    try {
      const id = await ecrireReception<string>("remarque", { id: remarque?.id ?? null, page, texte, genre, gravite, images, garder: gardees });
      if (remarque) {
        fini?.(id);
      } else {
        message = `${id} enregistrée.`;
        texte = "";
        images = [];
      }
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    } finally {
      enCours = false;
    }
  }
</script>

<form class="nouvelle-remarque" class:edition={!!remarque} bind:this={formulaire} onsubmit={enregistrer}>
  {#if remarque}<p class="discret">Modifier {remarque.id} : la modification sera enregistrée comme une nouvelle version, l'ancienne reste dans l'historique.</p>{/if}
  <textarea
    rows={3}
    bind:value={texte}
    onkeydown={(e) => e.key === "Enter" && (e.ctrlKey || e.metaKey) && enregistrer(e)}
    placeholder="Une remarque par carte. Colle tes captures avec Ctrl+V."
    aria-label="Remarque"
  ></textarea>
  <Captures bind:images bind:gardees bind:erreur principal={!remarque} {formulaire} />
  <div class="actions">
    <span class="bascule" role="group" aria-label="Genre">
      {#each Object.entries(GENRES) as [v, nom]}<button type="button" class:actif={genre === v} onclick={() => (genre = v)}>{nom}</button>{/each}
    </span>
    <span class="bascule" role="group" aria-label="Gravité">
      {#each Object.entries(GRAVITES) as [v, nom]}<button type="button" class:actif={gravite === v} onclick={() => (gravite = v)}>{nom}</button>{/each}
    </span>
    <button class="principal" type="submit" disabled={enCours || vide}>{remarque ? "Enregistrer la nouvelle version" : "Ajouter la remarque"}</button>
    {#if remarque}<button type="button" class="discret-bouton" onclick={() => fini?.(remarque!.id)}>Annuler</button>{/if}
    {#if message}<span class="succes">{message}</span>{/if}
  </div>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
</form>
