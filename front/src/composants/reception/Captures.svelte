<script lang="ts">
  // Ajout de captures (SPEC-23 R10) : tuile « ＋ Capture » qui ouvre une aide (raccourcis de capture par
  // système, coller, glisser, choisir un fichier ; « coller maintenant » quand le navigateur le permet).
  // Le collage est écouté sur toute la page : il va au formulaire qui a le focus, sinon au formulaire
  // principal (`principal`). Sur téléphone (REC-092), la tuile prend directement l'image du presse-papiers
  // quand le navigateur le permet (https) ; sinon l'aide montre une zone où coller par appui long, ce qui
  // marche aussi en http, et les raccourcis de capture du téléphone.
  import { onMount } from "svelte";

  let {
    images = $bindable([]),
    gardees = $bindable([]),
    principal = false,
    formulaire,
    erreur = $bindable(""),
  }: { images?: string[]; gardees?: string[]; principal?: boolean; formulaire: HTMLElement | undefined; erreur?: string } = $props();

  /** Aide de capture ouverte (raccourcis, coller, glisser, choisir un fichier). */
  let aide = $state(false);
  /** Fichier glissé au-dessus de la tuile, pour la surligner. */
  let survol = $state(false);
  /** Champ de fichier caché, ouvert par « Choisir un fichier ». */
  let fichier = $state<HTMLInputElement | null>(null);
  /** Lecture directe du presse-papiers possible : seulement en contexte sécurisé (https, localhost). */
  const lecturePossible = typeof navigator !== "undefined" && !!navigator.clipboard?.read && window.isSecureContext;
  /** Écran tactile sans souris : téléphone ou tablette. */
  const tactile = typeof matchMedia !== "undefined" && matchMedia("(hover: none) and (pointer: coarse)").matches;
  /** Système de l'utilisateur, pour montrer ses raccourcis de capture.
   * @relu 2026-10-24 cf5af999
   */
  const systeme = /iPhone|iPad/.test(navigator.userAgent) ? "ios" : /Android/.test(navigator.userAgent) ? "android" : /Mac/.test(navigator.platform) ? "mac" : /Win/.test(navigator.platform) ? "windows" : "linux";
  /** Raccourcis de capture d'écran par système, téléphones compris : touches et ce qu'elles capturent.
   * @relu 2026-10-24 d79d8e5c
   */
  const RACCOURCIS: Record<string, [string, string][]> = {
    windows: [["Win + Maj + S", "zone de l'écran"], ["Impr. écran", "écran entier"]],
    mac: [["Cmd + Ctrl + Maj + 4", "zone, dans le presse-papiers"], ["Cmd + Ctrl + Maj + 3", "écran entier"]],
    linux: [["Maj + Impr. écran", "zone de l'écran"], ["Impr. écran", "écran entier"]],
    android: [["Volume bas + Marche", "écran entier ; l'aperçu propose « Copier »"]],
    ios: [["Côté + Volume haut", "écran entier ; touche l'aperçu, puis Partager › Copier"]],
  };
  /** Tuile « Capture » : sur téléphone, lit d'abord le presse-papiers ; sinon (ou si rien n'est lu) ouvre l'aide. */
  async function tuile() {
    if (aide) {
      aide = false;
      return;
    }
    if (tactile && lecturePossible && (await collerMaintenant(true))) return;
    aide = true;
  }

  /** Ajoute des images en data URL : PNG, JPEG, GIF ou WebP de 10 Mo au plus, les autres sont refusées avec leur nom. */
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

  /**
   * « Coller maintenant » : lit les images du presse-papiers sans Ctrl+V et renvoie leur nombre ; un refus du navigateur
   * renvoie vers Ctrl+V. `discret` : rien n'est signalé (la tuile du téléphone ouvre alors l'aide).
   */
  async function collerMaintenant(discret = false): Promise<number> {
    try {
      const elements = await navigator.clipboard.read();
      const blobs: File[] = [];
      for (const el of elements)
        for (const t of el.types.filter((t) => t.startsWith("image/"))) blobs.push(new File([await el.getType(t)], "capture", { type: t }));
      if (!blobs.length && !discret) erreur = "Le presse-papiers ne contient pas d'image : fais d'abord ta capture.";
      lire(blobs);
      return blobs.length;
    } catch {
      if (!discret) erreur = "Le navigateur refuse la lecture directe : utilise Ctrl+V.";
      return 0;
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
    <button type="button" class="tuile-capture" onclick={tuile} aria-expanded={aide}>
      <span class="plus">＋</span><span>Capture</span><small>{tactile ? "presse-papiers" : "Ctrl+V"}</small>
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
          {#if tactile}
            <li>
              Reviens ici, appuie longuement dans la zone et choisis « Coller » :
              <div class="zone-collage" contenteditable="true" role="textbox" tabindex="0" aria-label="Zone où coller la capture" oninput={(e) => (e.currentTarget.textContent = "")}></div>
            </li>
          {:else}
            <li>Reviens ici et colle : <kbd>Ctrl + V</kbd>, n'importe où sur la page.</li>
          {/if}
        </ol>
        <p class="discret">{tactile ? "Ou choisis une capture dans tes images." : "Ou glisse une image sur la carte."}</p>
        <div class="actions">
          {#if lecturePossible}<button type="button" class="principal" onclick={() => collerMaintenant()}>Coller maintenant</button>{/if}
          <button type="button" onclick={() => fichier?.click()}>Choisir un fichier…</button>
        </div>
      </div>
    {/if}
    <input bind:this={fichier} type="file" accept="image/png,image/jpeg,image/gif,image/webp" multiple hidden onchange={(e) => lire((e.currentTarget as HTMLInputElement).files ?? [])} />
  </span>
</div>

<style>
  .zone-collage {
    min-height: 56px;
    margin: 6px 0;
    border: 2px dashed var(--accent);
    border-radius: var(--rayon);
    background: var(--accent-fond);
  }
</style>
