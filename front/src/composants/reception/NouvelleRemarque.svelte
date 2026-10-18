<script lang="ts">
  // Saisie d'une remarque : texte, genre, gravité, captures collées (Ctrl+V) ou déposées. Une carte = une
  // remarque ; Ctrl+Entrée enregistre.
  import { ecrireReception, GENRES, GRAVITES } from "../../lib/reception.svelte";

  let { page }: { page: string } = $props();
  let texte = $state("");
  let genre = $state("ergonomie");
  let gravite = $state("genant");
  let images = $state<string[]>([]);
  let erreur = $state("");
  let enCours = $state(false);
  let message = $state("");

  function lire(fichiers: Iterable<File>) {
    for (const f of fichiers) {
      if (!/^image\/(png|jpeg|gif|webp)$/.test(f.type)) {
        erreur = `${f.name || "fichier"} : seules les images PNG, JPEG, GIF et WebP sont acceptées`;
        continue;
      }
      if (f.size > 10 * 1024 * 1024) {
        erreur = `${f.name || "image"} : 10 Mo au plus`;
        continue;
      }
      const l = new FileReader();
      l.onload = () => (images = [...images, String(l.result)]);
      l.readAsDataURL(f);
    }
  }

  function coller(e: ClipboardEvent) {
    const f = [...(e.clipboardData?.files ?? [])];
    if (f.length) {
      e.preventDefault();
      erreur = "";
      lire(f);
    }
  }

  function deposer(e: DragEvent) {
    e.preventDefault();
    erreur = "";
    lire(e.dataTransfer?.files ?? []);
  }

  async function enregistrer(e?: Event) {
    e?.preventDefault();
    if (!texte.trim() && !images.length) return;
    enCours = true;
    erreur = message = "";
    try {
      const id = await ecrireReception<string>("remarque", { page, texte, genre, gravite, images });
      message = `${id} enregistrée.`;
      texte = "";
      images = [];
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    } finally {
      enCours = false;
    }
  }
</script>

<form class="nouvelle-remarque" onsubmit={enregistrer} ondragover={(e) => e.preventDefault()} ondrop={deposer}>
  <textarea
    rows={3}
    bind:value={texte}
    onpaste={coller}
    onkeydown={(e) => e.key === "Enter" && (e.ctrlKey || e.metaKey) && enregistrer(e)}
    placeholder="Une remarque par carte. Colle tes captures ici (Ctrl+V) ou glisse-les sur la carte."
    aria-label="Remarque"
  ></textarea>
  {#if images.length}
    <div class="captures">
      {#each images as src, i}
        <span class="capture apercu-capture">
          <img {src} alt="capture {i + 1}" />
          <button type="button" onclick={() => (images = images.filter((_, j) => j !== i))} aria-label="Retirer">×</button>
        </span>
      {/each}
    </div>
  {/if}
  <div class="actions">
    <span class="bascule" role="group" aria-label="Genre">
      {#each Object.entries(GENRES) as [v, nom]}<button type="button" class:actif={genre === v} onclick={() => (genre = v)}>{nom}</button>{/each}
    </span>
    <span class="bascule" role="group" aria-label="Gravité">
      {#each Object.entries(GRAVITES) as [v, nom]}<button type="button" class:actif={gravite === v} onclick={() => (gravite = v)}>{nom}</button>{/each}
    </span>
    <button class="principal" type="submit" disabled={enCours || (!texte.trim() && !images.length)}>Ajouter la remarque</button>
    {#if message}<span class="succes">{message}</span>{/if}
  </div>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
</form>
