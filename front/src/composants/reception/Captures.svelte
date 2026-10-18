<script lang="ts">
  // Ajout de captures (SPEC-23 R10) : tuile « ＋ Capture » qui ouvre une aide (raccourcis de capture par
  // système, coller, glisser, choisir un fichier ; « coller maintenant » quand le navigateur le permet).
  // Le collage est écouté sur toute la page : il va au formulaire qui a le focus, sinon au formulaire
  // principal (`principal`).
  import { onMount } from "svelte";

  let {
    images = $bindable([]),
    gardees = $bindable([]),
    principal = false,
    formulaire,
    erreur = $bindable(""),
  }: { images?: string[]; gardees?: string[]; principal?: boolean; formulaire: HTMLElement | undefined; erreur?: string } = $props();

  let aide = $state(false);
  let survol = $state(false);
  let fichier = $state<HTMLInputElement | null>(null);
  // Lecture directe du presse-papiers : seulement en contexte sécurisé (https, localhost).
  const lecturePossible = typeof navigator !== "undefined" && !!navigator.clipboard?.read && window.isSecureContext;
  const systeme = /Mac|iPhone|iPad/.test(navigator.platform) ? "mac" : /Win/.test(navigator.platform) ? "windows" : "linux";
  const RACCOURCIS: Record<string, [string, string][]> = {
    windows: [["Win + Maj + S", "zone de l'écran"], ["Impr. écran", "écran entier"]],
    mac: [["Cmd + Ctrl + Maj + 4", "zone, dans le presse-papiers"], ["Cmd + Ctrl + Maj + 3", "écran entier"]],
    linux: [["Maj + Impr. écran", "zone de l'écran"], ["Impr. écran", "écran entier"]],
  };

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
      aide = false;
    }
  }

  async function collerMaintenant() {
    try {
      const elements = await navigator.clipboard.read();
      const blobs: File[] = [];
      for (const el of elements)
        for (const t of el.types.filter((t) => t.startsWith("image/"))) blobs.push(new File([await el.getType(t)], "capture", { type: t }));
      if (!blobs.length) erreur = "Le presse-papiers ne contient pas d'image : fais d'abord ta capture.";
      lire(blobs);
    } catch {
      erreur = "Le navigateur refuse la lecture directe : utilise Ctrl+V.";
    }
  }

  onMount(() => {
    const surCollage = (e: ClipboardEvent) => {
      const f = [...(e.clipboardData?.files ?? [])].filter((x) => x.type.startsWith("image/"));
      if (!f.length || e.defaultPrevented || !formulaire) return;
      const actif = document.activeElement;
      const dansUnFormulaire = actif?.closest?.(".nouvelle-remarque");
      // Le formulaire qui a le focus prend la capture ; sinon le formulaire principal de la page.
      if (dansUnFormulaire ? dansUnFormulaire !== formulaire : !principal) return;
      e.preventDefault();
      erreur = "";
      lire(f);
    };
    window.addEventListener("paste", surCollage);
    return () => window.removeEventListener("paste", surCollage);
  });
</script>

<div
  class="captures"
  class:survol
  role="group"
  aria-label="Captures"
  ondragover={(e) => { e.preventDefault(); survol = true; }}
  ondragleave={() => (survol = false)}
  ondrop={(e) => { e.preventDefault(); survol = false; erreur = ""; lire(e.dataTransfer?.files ?? []); }}
>
  {#each gardees as src, i (src)}
    <span class="capture apercu-capture">
      <img src={`/api/image?chemin=${encodeURIComponent(src)}`} alt="capture gardée {i + 1}" />
      <button type="button" onclick={() => (gardees = gardees.filter((x) => x !== src))} aria-label="Retirer">×</button>
    </span>
  {/each}
  {#each images as src, i}
    <span class="capture apercu-capture">
      <img {src} alt="capture {i + 1}" />
      <button type="button" onclick={() => (images = images.filter((_, j) => j !== i))} aria-label="Retirer">×</button>
    </span>
  {/each}
  <span class="ajout-capture">
    <button type="button" class="tuile-capture" onclick={() => (aide = !aide)} aria-expanded={aide}>
      <span class="plus">＋</span><span>Capture</span><small>Ctrl+V</small>
    </button>
    {#if aide}
      <div class="aide-capture" role="dialog" aria-label="Ajouter une capture">
        <button type="button" class="fermer" onclick={() => (aide = false)} aria-label="Fermer">×</button>
        <strong>Ajouter une capture</strong>
        <ol>
          <li>
            Capture l'écran :
            <ul>{#each RACCOURCIS[systeme] as [touches, quoi]}<li><kbd>{touches}</kbd> {quoi}</li>{/each}</ul>
          </li>
          <li>Reviens ici et colle : <kbd>Ctrl + V</kbd>, n'importe où sur la page.</li>
        </ol>
        <p class="discret">Ou glisse une image sur la carte.</p>
        <div class="actions">
          {#if lecturePossible}<button type="button" class="principal" onclick={collerMaintenant}>Coller maintenant</button>{/if}
          <button type="button" onclick={() => fichier?.click()}>Choisir un fichier…</button>
        </div>
      </div>
    {/if}
    <input bind:this={fichier} type="file" accept="image/png,image/jpeg,image/gif,image/webp" multiple hidden onchange={(e) => lire((e.currentTarget as HTMLInputElement).files ?? [])} />
  </span>
</div>
