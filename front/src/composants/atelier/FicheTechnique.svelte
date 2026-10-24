<script lang="ts">
  // Fiche d'une technique pour l'atelier affiché (EF-178, ADR-0019, REC-098) : l'encart ⚙ montre le texte que
  // le LLM reçoit à cette étape (sous-étapes, rôle, consignes, règles, exemple) ; l'enregistrer en fait la
  // fiche de cet atelier seulement (`<dossier>/.techniques/<id>.md`), « revenir à l'origine » la supprime.
  // Une fiche qui ne s'analyse plus est refusée par le serveur, avec la raison.
  // @req EF-178
  import { atelier, ecrire } from "../../lib/atelier.svelte";
  import { signaler } from "../../lib/edition.svelte";

  let { etape, propre }: { etape: string; propre: boolean } = $props();
  /** Texte de la fiche pour cet atelier, chargé à l'ouverture de l'encart. */
  let texte = $state<string | null>(null);
  /** Texte tel que lu, pour savoir s'il a changé. */
  let lu = $state("");
  /** Erreur de lecture ou d'écriture (fiche invalide). */
  let erreur = $state("");

  /** Lit la fiche de l'atelier (sa copie corrigée, sinon l'origine). */
  async function charger() {
    erreur = "";
    const q = new URLSearchParams({ parcours: atelier.parcours, etape });
    const r = await fetch(`/api/atelier/fiche?${q}`);
    const v = await r.json().catch(() => ({}));
    if (!r.ok) return void (erreur = v.erreur ?? `${r.status}`);
    texte = lu = v.texte;
  }
  /** Enregistre la fiche corrigée, ou (`null`) revient à celle d'origine. */
  async function enregistrer(t: string | null) {
    erreur = "";
    if (t === null && !confirm("Revenir à la fiche d'origine pour cet atelier ? La copie corrigée sera supprimée.")) return;
    try {
      await ecrire("fiche", { etape, texte: t });
      signaler(t === null ? "Fiche d'origine rétablie." : "Fiche corrigée pour cet atelier.");
      await charger();
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

<details class="prompt-systeme" ontoggle={(e) => e.currentTarget.open && texte === null && charger()}>
  <summary>
    <span class="engrenage" aria-hidden="true">⚙</span> Fiche de la technique (consignes du LLM)
    {#if propre}<span class="pastille statut-proposé">corrigée pour cet atelier</span>{:else}<span class="pastille discret">texte d'origine</span>{/if}
  </summary>
  <p class="discret">Ce que le LLM reçoit à cette étape. Une correction vaut pour cet atelier seulement ; les sections <code>### &lt;sous-étape&gt;</code> de « Consignes » sont les consignes de chaque sous-étape.</p>
  {#if texte !== null}
    <textarea class="mono" rows={18} bind:value={texte} aria-label="Texte de la fiche {etape}"></textarea>
    <div class="actions">
      <button class="principal" type="button" onclick={() => enregistrer(texte)} disabled={texte === lu}>Enregistrer pour cet atelier</button>
      {#if propre}<button type="button" class="discret-bouton" onclick={() => enregistrer(null)}>Revenir à l'origine</button>{/if}
    </div>
  {/if}
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
</details>

<style>
  .prompt-systeme {
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    padding: 2px 12px 8px;
    margin: 10px 0;
    background: var(--fond-2);
    font-size: 14px;
  }
  .prompt-systeme > summary {
    cursor: pointer;
    padding: 8px 0 2px;
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
    color: var(--texte-2);
  }
  .engrenage {
    color: var(--accent);
  }
  textarea {
    width: 100%;
    font-size: 12.5px;
    line-height: 1.45;
  }
</style>
