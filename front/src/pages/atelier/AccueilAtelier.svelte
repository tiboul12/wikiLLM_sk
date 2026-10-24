<script lang="ts">
  // Accueil du Design thinking (EF-177, REC-091) : on y arrive avant d'entrer dans un atelier. Les ateliers du
  // projet en cartes (numéro, titre, version, statut, avancement), la création d'un atelier, et l'état des
  // clarifications ; un atelier s'ouvre sur sa page (`/atelier?parcours=…`).
  // @ecrans /atelier Design thinking
  // @req EF-177
  import { atelier, ecrire, STATUTS_ATELIER } from "../../lib/atelier.svelte";
  import { aller, url } from "../../lib/routeur.svelte";

  /** Ateliers du projet, le plus récent en premier. */
  const ateliers = $derived([...(atelier.etat?.tous_parcours ?? [])].sort((a, b) => b.numero - a.numero));
  /** Clarifications encore ouvertes. */
  const ouvertes = $derived(atelier.etat?.clarifications.filter((c) => c.frontmatter.etat === "ouverte").length ?? 0);

  /** Formulaire de création ouvert. */
  let ajout = $state(false);
  /** Titre du nouvel atelier. */
  let titre = $state("");
  /** Version que documente le nouvel atelier. */
  let version = $state("");
  /** Erreur de création. */
  let erreur = $state("");
  /** Crée l'atelier et l'ouvre. */
  async function creer(e: SubmitEvent) {
    e.preventDefault();
    erreur = "";
    try {
      const pid = await ecrire<string>("parcours", { titre, version });
      ajout = false;
      titre = version = "";
      aller(url.atelier(pid));
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }
</script>

<h1>Design thinking</h1>
<p class="discret">Définir le besoin selon le framework Fast Design Thinking : un atelier par sujet ou par version. Le LLM ({atelier.etat?.fournisseur}) lit le projet et propose ; tu gardes, corriges ou rejettes.</p>

<div class="cartes-ateliers">
  {#each ateliers as p (p.id)}
    <a class="carte-atelier statut-{p.statut}" href={url.atelier(p.id)}>
      <span class="entete-carte"><span class="numero-atelier">n° {p.numero}</span><span class="pastille"><span class="statut-atelier statut-{p.statut}"></span> {STATUTS_ATELIER[p.statut] ?? p.statut}</span></span>
      <strong>{p.titre}</strong>
      <span class="discret">version {p.version || "?"} · {p.faites} / {p.total} étapes</span>
      <span class="jauge"><span style="width: {p.total ? (100 * p.faites) / p.total : 0}%"></span></span>
    </a>
  {/each}
  {#if ajout}
    <form class="carte-atelier nouvel-atelier formulaire" onsubmit={creer}>
      <label><span>Titre</span><input bind:value={titre} required placeholder="Appli météo" /></label>
      <label><span>Version documentée</span><input bind:value={version} placeholder="v2" /></label>
      <div class="actions"><button class="principal" type="submit">Créer</button><button type="button" onclick={() => (ajout = false)}>Annuler</button></div>
      {#if erreur}<p class="erreur">{erreur}</p>{/if}
    </form>
  {:else}
    <button class="carte-atelier nouvel-atelier" onclick={() => (ajout = true)}><span class="plus">＋</span> Nouvel atelier</button>
  {/if}
</div>

<h2>Clarifications</h2>
<p><a href={url.clarifications()}>{atelier.etat?.clarifications.length ?? 0} clarification(s)</a>{ouvertes ? ` · ${ouvertes} ouverte(s)` : ""} : les points à arbitrer, posés depuis un besoin, une exigence ou une fiche de l'atelier.</p>

<style>
  .cartes-ateliers {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
    margin: 12px 0 24px;
  }
  .carte-atelier {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 14px 16px;
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    background: var(--fond-2);
    color: var(--texte);
    text-align: left;
    font: inherit;
  }
  a.carte-atelier:hover {
    border-color: var(--accent);
    text-decoration: none;
  }
  .entete-carte {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
  }
  .numero-atelier {
    color: var(--texte-2);
    font-family: var(--mono);
  }
  .carte-atelier strong {
    font-size: 16px;
  }
  .nouvel-atelier {
    border-style: dashed;
    cursor: pointer;
    justify-content: center;
    align-items: center;
    color: var(--accent);
    min-height: 110px;
  }
  form.nouvel-atelier {
    align-items: stretch;
    cursor: default;
    color: var(--texte);
  }
  .plus {
    font-size: 22px;
  }
</style>
