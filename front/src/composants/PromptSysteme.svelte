<script lang="ts">
  // Prompt système d'une page, sur la page qui s'en sert (EF-176, REC-104.3) : l'encart repliable du skill
  // que wikik donne à son LLM ou à l'agent pour cette tâche (structure d'un type de document, clarification,
  // traitement de la réception, technique d'un atelier). Même édition que dans les Paramètres : enregistrer
  // l'écrit dans `.claude/skills/<nom>/SKILL.md` (ADR-0017), le texte d'origine se rétablit. Écriture réservée
  // au propriétaire (ADR-0013) ; absent du site exporté.
  // @req EF-176
  // @relu 2026-10-24 def7f184
  import { terminal, chargerParametres, ecrireSkill } from "../lib/terminal.svelte";
  import { statique } from "../lib/statique";

  let { nom, libelle = "Prompt système", ouvert = false }: { nom: string; libelle?: string; ouvert?: boolean } = $props();

  if (!statique && !terminal.skills.length) chargerParametres();
  /** Le skill affiché, tel que le serveur le décrit ; absent tant que les paramètres ne sont pas chargés. */
  const skill = $derived(terminal.skills.find((s) => s.nom === nom));
  /** Texte en cours d'édition ; `null` tant qu'on n'y a pas touché (le texte du serveur s'affiche). */
  let brouillon = $state<string | null>(null);
  /** Confirmation ou erreur du dernier enregistrement. */
  let message = $state("");
  /** Texte affiché dans la zone d'édition. */
  const texte = $derived(brouillon ?? skill?.texte ?? "");

  /** Écrit le texte dans le `SKILL.md` du projet. */
  async function enregistrer() {
    message = "";
    try {
      await ecrireSkill(nom, texte);
      brouillon = null;
      message = "Enregistré dans " + (skill?.chemin ?? ".claude/skills/");
    } catch (x) {
      message = "Erreur : " + String(x instanceof Error ? x.message : x);
    }
  }
  /** Revient au texte livré avec wikik, après confirmation : le fichier du projet est supprimé. */
  async function retablir() {
    if (!skill || !confirm(`Revenir au texte d'origine de « ${skill.titre} » ? Le fichier ${skill.chemin} sera supprimé.`)) return;
    await ecrireSkill(nom, null);
    brouillon = null;
    message = "Texte d'origine rétabli.";
  }
</script>

{#if !statique && skill}
  <details class="prompt-systeme" id="skill-{nom}" open={ouvert}>
    <summary>
      <span class="engrenage" aria-hidden="true">⚙</span> {libelle} : <strong>{skill.titre}</strong>
      {#if skill.modifie}<span class="pastille statut-proposé">modifié</span>{:else if skill.installe}<span class="pastille">dans le projet</span>{:else}<span class="pastille discret">texte d'origine</span>{/if}
    </summary>
    <p class="discret">{skill.role}</p>
    <fieldset disabled={!terminal.proprietaire}>
      <textarea class="mono" rows={14} value={texte} oninput={(e) => (brouillon = e.currentTarget.value)} aria-label="Texte du skill {nom}"></textarea>
      <div class="actions">
        <button class="principal" type="button" onclick={enregistrer} disabled={brouillon === null && skill.installe}>Enregistrer</button>
        {#if skill.installe}<button type="button" class="discret-bouton" onclick={retablir}>Revenir au texte d'origine</button>{/if}
        <small class="discret">{skill.chemin}</small>
        {#if message}<span class={message.startsWith("Erreur") ? "erreur" : "succes"}>{message}</span>{/if}
      </div>
    </fieldset>
  </details>
{/if}

<style>
  .prompt-systeme {
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    padding: 2px 12px;
    margin: 10px 0;
    background: var(--fond-2);
    font-size: 14px;
  }
  .prompt-systeme > summary {
    cursor: pointer;
    padding: 8px 0;
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
    color: var(--texte-2);
  }
  .prompt-systeme > summary strong {
    color: var(--texte);
    font-weight: 600;
  }
  .engrenage {
    color: var(--accent);
  }
  fieldset {
    border: none;
    padding: 0 0 10px;
    margin: 0;
  }
  textarea {
    width: 100%;
    font-size: 12.5px;
    line-height: 1.45;
  }
</style>
