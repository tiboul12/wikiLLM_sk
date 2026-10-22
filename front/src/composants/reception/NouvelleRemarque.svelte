<script lang="ts">
  // Saisie d'une remarque : texte, genre, gravité, captures. Une carte = une remarque ; Ctrl+Entrée
  // enregistre. Avec `remarque`, la saisie en écrit la version suivante (SPEC-23 R8).
  import type { Remarque } from "../../types/Remarque";
  import { ecrireReception, GENRES, GRAVITES } from "../../lib/reception.svelte";
  import Captures from "./Captures.svelte";

  let { page, remarque = null, fini }: { page: string; remarque?: Remarque | null; fini?: (id: string) => void } = $props();
  // svelte-ignore state_referenced_locally
  /** Texte de la remarque ; prérempli avec la version modifiée. */
  let texte = $state(remarque?.texte ?? "");
  // svelte-ignore state_referenced_locally
  /** Genre de la remarque (ergonomie par défaut). */
  let genre = $state(remarque?.genre ?? "ergonomie");
  // svelte-ignore state_referenced_locally
  /** Gravité de la remarque (gênant par défaut). */
  let gravite = $state(remarque?.gravite ?? "genant");
  /** Nouvelles captures, en data URL, envoyées avec la remarque. */
  let images = $state<string[]>([]);
  // svelte-ignore state_referenced_locally
  /** Captures de la version précédente qu'on garde dans la nouvelle. */
  let gardees = $state<string[]>(remarque ? [...remarque.images] : []);
  /** Erreur d'enregistrement. */
  let erreur = $state("");
  /** Enregistrement en cours. */
  let enCours = $state(false);
  /** Confirmation après l'ajout d'une remarque. */
  let message = $state("");
  /** Formulaire, pour enregistrer au Ctrl+Entrée depuis n'importe quel champ. */
  let formulaire = $state<HTMLFormElement>();
  /** Rien à enregistrer : ni texte ni capture. */
  const vide = $derived(!texte.trim() && !images.length && !gardees.length);

  /**
   * Enregistre la remarque, ou sa nouvelle version ; une nouvelle remarque vide le formulaire pour la suivante,
   * une modification rend la main avec le nouvel identifiant.
   */
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
