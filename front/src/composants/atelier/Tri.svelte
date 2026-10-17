<script lang="ts">
  // Bilan : tri des besoins par glisser-déposer en priorités MoSCoW, catégorie au choix.
  import { deType, ecrire } from "../../lib/atelier.svelte";
  import { url } from "../../lib/routeur.svelte";

  const COLONNES: [string, string][] = [
    ["must", "Must : indispensable"],
    ["should", "Should : important"],
    ["could", "Could : souhaitable"],
    ["wont", "Won't : pas cette fois"],
  ];
  const CATEGORIES = ["fonctionnel", "technique", "ergonomique", "réglementaire", "économique"];
  const besoins = $derived(deType("besoin"));
  let tri = $state<{ id: string; titre: string; priorite: string; categorie: string }[]>([]);
  let charge = $state("");
  $effect(() => {
    const cle = besoins.map((b) => b.id + b.frontmatter.priorite + b.frontmatter.categorie).join();
    if (cle === charge) return;
    charge = cle;
    const ordre = (b: (typeof besoins)[number]) => Number(b.frontmatter.ordre ?? 1e9);
    tri = [...besoins]
      .sort((a, b) => ordre(a) - ordre(b))
      .map((b) => ({ id: b.id, titre: b.titre, priorite: String(b.frontmatter.priorite || "should"), categorie: String(b.frontmatter.categorie || "fonctionnel") }));
  });
  let glisse = $state<string | null>(null);
  let message = $state("");
  let erreur = $state("");

  function deposer(priorite: string, avant: string | null) {
    if (!glisse) return;
    const b = tri.find((x) => x.id === glisse)!;
    const reste = tri.filter((x) => x.id !== glisse);
    b.priorite = priorite;
    const i = avant ? reste.findIndex((x) => x.id === avant) : reste.length;
    reste.splice(i < 0 ? reste.length : i, 0, b);
    tri = reste;
    glisse = null;
    message = "";
  }

  async function enregistrer() {
    erreur = message = "";
    try {
      await ecrire("saisir", { etape: "bilan", sous_etape: "tri", donnees: { besoins: tri.map((b, ordre) => ({ ...b, ordre })) } });
      message = "Tri enregistré.";
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }
</script>

{#if !tri.length}
  <p class="discret">Aucun besoin gardé pour l'instant.</p>
{:else}
  <div class="tableau-tri">
    {#each COLONNES as [p, nom]}
      <div class="colonne-tri" role="list" ondragover={(e) => e.preventDefault()} ondrop={() => deposer(p, null)}>
        <h4>{nom}</h4>
        {#each tri.filter((b) => b.priorite === p) as b (b.id)}
          <div
            class="carte-besoin"
            role="listitem"
            draggable="true"
            ondragstart={() => (glisse = b.id)}
            ondragover={(e) => e.preventDefault()}
            ondrop={(e) => { e.stopPropagation(); deposer(p, b.id); }}
          >
            <a class="id" href={url.document(b.id)}>{b.id}</a>
            <span>{b.titre}</span>
            <select bind:value={b.categorie} aria-label="Catégorie">{#each CATEGORIES as c}<option>{c}</option>{/each}</select>
            <select bind:value={b.priorite} aria-label="Priorité">{#each COLONNES as [v]}<option>{v}</option>{/each}</select>
          </div>
        {/each}
      </div>
    {/each}
  </div>
  <div class="actions">
    <button class="principal" onclick={enregistrer}>Enregistrer le tri</button>
    {#if message}<span class="succes">{message}</span>{/if}
  </div>
{/if}
{#if erreur}<p class="erreur">{erreur}</p>{/if}
