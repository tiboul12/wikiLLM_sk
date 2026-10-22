<script lang="ts">
  // Barre d'activité, à la VS Code (EF-131) : une icône par section, toujours visible, avec un badge pour
  // ce qui attend (remarques ouvertes en Réception, clarifications ouvertes dans le Design thinking,
  // décisions attendues en Projet) ; en bas les outils
  // (Code, avec son graphe : EF-150), les paramètres et, pour le propriétaire, l'arrêt du serveur
  // (ADR-0014). Sur téléphone, elle passe en barre du bas.
  import { SECTIONS, OUTILS, NOMS, type Section } from "../lib/sections.svelte";
  import { url } from "../lib/routeur.svelte";
  import { atelier } from "../lib/atelier.svelte";
  import { reception, actuelles } from "../lib/reception.svelte";
  import Icone from "./Icone.svelte";
  import { statique } from "../lib/statique";
  import { terminal, arreterServeur } from "../lib/terminal.svelte";

  async function arreter() {
    if (confirm("Arrêter le serveur wikik de ce projet ? Il faudra relancer « wikik » dans le projet pour le rouvrir.")) await arreterServeur();
  }

  let { active }: { active: Section | null } = $props();
  const LIENS: Record<Section, () => string> = {
    accueil: url.accueil,
    produit: url.produit,
    projet: url.projet,
    atelier: () => url.atelier(atelier.parcours),
    reception: url.reception,
    parametres: url.parametres,
  };
  const badge = (s: Section): number => {
    if (s === "reception") return reception.etat ? actuelles().filter((r) => r.statut === "ouverte").length : 0;
    if (s === "atelier") return atelier.etat?.clarifications.filter((c) => c.frontmatter.etat === "ouverte").length ?? 0;
    return 0;
  };
</script>

<nav class="barre-activite" aria-label="Sections">
  {#each SECTIONS as s (s)}
    {@const n = badge(s)}
    <a href={LIENS[s]()} class:actif={active === s} title={NOMS[s]} aria-current={active === s ? "page" : undefined}>
      <Icone nom={s} />
      <span class="libelle">{NOMS[s]}</span>
      {#if n}<span class="badge" title="{n} en attente">{n}</span>{/if}
    </a>
  {/each}
  <span class="espace"></span>
  {#each OUTILS as s (s)}
    <a href={LIENS[s]()} class:actif={active === s} title={NOMS[s]} aria-current={active === s ? "page" : undefined}>
      <Icone nom={s} />
      <span class="libelle">{NOMS[s]}</span>
    </a>
  {/each}
  {#if !statique}<a href={url.parametres()} class:actif={active === "parametres"} title="Paramètres" class="bas"><Icone nom="parametres" /><span class="libelle">Paramètres</span></a>{/if}
  {#if terminal.proprietaire}<button class="arret" onclick={arreter} title="Arrêter le serveur (déconnexion)"><Icone nom="arret" /><span class="libelle">Arrêter</span></button>{/if}
</nav>
