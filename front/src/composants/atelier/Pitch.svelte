<script lang="ts">
  // Écran du pitch : description libre, brouillon proposé par le LLM, puis formulaire en six champs.
  import type { SousEtape } from "../../types/SousEtape";
  import { artefact, champ, ecrire } from "../../lib/atelier.svelte";
  import Propositions from "./Propositions.svelte";

  let { se, mode }: { se: SousEtape; mode: string } = $props();
  /** Six champs du pitch : clé enregistrée, libellé, aide affichée dans le champ. */
  const CHAMPS: [string, string, string][] = [
    ["probleme", "Problème", "Quel problème, pour qui, aujourd'hui ?"],
    ["cible", "Pour qui", "Les personnes concernées"],
    ["valeur", "Valeur", "Ce que le projet change pour elles"],
    ["contraintes", "Contraintes", "Techniques, budget, délais, réglementaires"],
    ["criteres_succes", "Critères de succès", "Mesurables si possible"],
    ["hors_sujet", "Hors sujet", "Ce que le projet ne fera pas"],
  ];
  /** Pitch enregistré du projet (artefact `PITCH`). */
  const pitch = $derived(artefact("PITCH"));
  /** Valeurs du formulaire, par champ. */
  let f = $state<Record<string, string>>({});
  /** Empreinte du pitch chargé : le formulaire n'est rechargé que si le pitch enregistré change, pas pendant la saisie. */
  let charge = $state("");
  // Le formulaire reprend le pitch enregistré à chaque nouvelle version.
  $effect(() => {
    const cle = JSON.stringify(pitch?.frontmatter ?? {});
    if (cle === charge) return;
    charge = cle;
    f = Object.fromEntries(["titre", "description", ...CHAMPS.map((c) => c[0])].map((k) => [k, champ(pitch, k)]));
  });
  /** Confirmation d'enregistrement. */
  let message = $state("");
  /** Erreur d'enregistrement. */
  let erreur = $state("");

  /** Enregistre le pitch tel que saisi. */
  async function enregistrer(e: SubmitEvent) {
    e.preventDefault();
    erreur = message = "";
    try {
      await ecrire("saisir", { etape: "pitch", sous_etape: "pitch", donnees: f });
      message = "Pitch enregistré.";
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

<form class="formulaire pitch" onsubmit={enregistrer}>
  <label><span>Nom du projet</span><input bind:value={f.titre} placeholder="Un nom court" /></label>
  <label>
    <span>Description libre</span>
    <textarea rows={5} bind:value={f.description} placeholder="Raconte le projet comme il te vient : le LLM en tirera un brouillon de pitch."></textarea>
  </label>
  {#each CHAMPS as [k, nom, aide]}
    <label><span>{nom}</span><textarea rows={2} bind:value={f[k]} placeholder={aide}></textarea></label>
  {/each}
  <div class="actions">
    <button class="principal" type="submit" disabled={!f.titre?.trim()}>Enregistrer le pitch</button>
    {#if message}<span class="succes">{message}</span>{/if}
  </div>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
</form>

{#if f.description?.trim()}
  <h4>Brouillon du LLM</h4>
  <p class="discret">Le LLM part de la description ; ce qui manque est marqué « à préciser ». Garder le brouillon remplace le pitch.</p>
  <Propositions etape="pitch" {se} {mode} entree={f.description} />
{/if}
