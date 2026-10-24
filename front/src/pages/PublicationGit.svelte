<script lang="ts">
  // Publication git (REC-111, EF-183) : la procédure que l'agent suit pour publier une version (skill
  // `wikik-publication-git` : une branche par version, commits par lot vérifiés, réception, fusion, tag et
  // poussée avec ton accord), la situation du dépôt qui dit où l'on en est, et « Publier avec l'agent » qui lui
  // envoie le skill dans le terminal. Le skill se lit et se corrige sur place.
  // @req EF-183
  import type { Historique } from "../types/Historique";
  import { url } from "../lib/routeur.svelte";
  import { terminal, lancerSkill } from "../lib/terminal.svelte";
  import PromptSysteme from "../composants/PromptSysteme.svelte";

  /** Situation du dépôt (branche, à pousser, branches et fusion) ; `null` hors d'un dépôt git. */
  let situation = $state<Historique | null>(null);
  fetch("/api/git/historique?limite=1")
    .then((r) => (r.ok ? r.json() : null))
    .then((h) => (situation = h))
    .catch(() => {});
  /** Branche courante, telle que la principale la voit. */
  const courante = $derived(situation?.etat_branches.find((b) => b.nom === situation?.branche));
  /** Message après l'envoi du skill, ou erreur. */
  let message = $state("");
  /** Envoie le skill de publication à l'agent du terminal. */
  async function publier() {
    message = "";
    try {
      await lancerSkill("wikik-publication-git");
      message = "Skill de publication envoyé à l'agent, dans le terminal.";
    } catch (x) {
      message = "Erreur : " + String(x instanceof Error ? x.message : x);
    }
  }
</script>

<p class="fil"><a href={url.git()}>Git</a> · publication</p>
<h1>Publication</h1>
<p class="discret">Comment une version passe de sa branche à la branche principale : commits par lot vérifiés, poussée, réception par toi, puis fusion et tag. L'agent suit cette procédure quand tu lui demandes de publier.</p>

{#if situation}
  <ol class="etapes-publication">
    <li class:fait={!!situation.branche && situation.branche !== situation.principale}>Travailler sur la branche de la version : <strong>{situation.branche || "HEAD détaché"}</strong>{#if courante?.partie_de}{" "}(partie de {courante.partie_de}){/if}.</li>
    <li class:fait={situation.modifies === 0}>Tout commiter : {situation.modifies ? `${situation.modifies} fichier(s) non commité(s)` : "rien en attente"}.</li>
    <li class:fait={situation.poussee && situation.en_avance === 0}>Pousser : {!situation.poussee ? "la branche n'a jamais été poussée" : situation.en_avance ? `${situation.en_avance} commit(s) à pousser` : "branche poussée"}.</li>
    <li>Recevoir la version : <a href={url.traitement()}>valider les remarques traitées</a>.</li>
    <li class:fait={!!courante?.fusionnee}>Fusionner dans <code>{situation.principale || "la principale"}</code>, taguer, pousser : {courante ? (courante.fusionnee ? "fusionnée" : `${courante.avance} commit(s) d'avance, pas encore fusionnée`) : "—"}.</li>
  </ol>
{/if}

<div class="actions">
  {#if terminal.proprietaire}<button class="principal" onclick={publier}>Publier avec l'agent ↗</button>{/if}
  {#if message}<span class={message.startsWith("Erreur") ? "erreur" : "succes"}>{message}</span>{/if}
</div>
<PromptSysteme nom="wikik-publication-git" libelle="Skill de publication" ouvert />

<style>
  .etapes-publication {
    padding-left: 22px;
    line-height: 1.7;
  }
  .etapes-publication li.fait::marker {
    content: "✓  ";
    color: var(--vert);
  }
</style>
