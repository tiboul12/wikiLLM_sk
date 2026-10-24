<script lang="ts">
  // Affichage lisible d'une valeur de proposition : texte (identifiants, chemins, symboles et routes d'écran
  // cités deviennent des liens, REC-071), liste de pastilles liées, objet en champs.
  import Valeur from "./Valeur.svelte";
  import { url } from "../../lib/routeur.svelte";
  import { autoLien } from "../../lib/connus.svelte";
  import { lienRoute } from "../../lib/reception.svelte";
  import { rendre } from "../../lib/markdown";

  /** Lien d'un texte cité : identifiant, chemin ou symbole connu du projet, sinon route d'écran de la réception. */
  const lien = (t: string) => autoLien(t) ?? lienRoute(t);

  let { v, cle = "", ordre = [] }: { v: unknown; cle?: string; ordre?: string[] } = $props();
  /** Horizons d'une roadmap, dans l'ordre d'affichage. */
  const HORIZONS: [string, string][] = [["maintenant", "Maintenant"], ["ensuite", "Ensuite"], ["plus_tard", "Plus tard"]];
  /** Objet roadmap (maintenant / ensuite / plus tard), affiché en trois colonnes au lieu de champs. */
  const estRoadmap = (o: unknown) => !!o && typeof o === "object" && !Array.isArray(o) && HORIZONS.every(([k]) => k in (o as object));
  /** Texte qui a la forme d'un identifiant de document (`EF-012`, `B-003`), affiché en pastille liée. */
  const estId = (t: string) => /^[A-Z]+(-[A-Za-z0-9]+)+$/.test(t);
  /** Libellés lisibles des champs dont le nom technique ne se lit pas tel quel. */
  const LIBELLES: Record<string, string> = {
    criteres_succes: "critères de succès",
    hors_sujet: "hors sujet",
    plus_tard: "plus tard",
    enonce: "énoncé",
    critere: "critère d'acceptation",
    priorite: "priorité",
    categorie: "catégorie",
  };
  /** Champs principaux d'abord (le schéma arrive trié par ordre alphabétique), puis l'ordre donné, puis le reste. */
  const PRIORITE = ["titre", "piste", "verbatim", "qualification", "analogie", "domaine", "nom", "levier", "enonce", "critere",
    "description", "scenario", "personnage", "scene", "intrigue", "objectif", "recit", "justification", "principes",
    "categorie", "priorite", "genre", "vignette", "prompt", "parent", "besoins", "sources"];
  /** Rang d'affichage d'un champ. */
  const rang = (k: string) => {
    const i = PRIORITE.indexOf(k);
    return i >= 0 ? i : 100 + (ordre.includes(k) ? ordre.indexOf(k) : 99);
  };
  /** Champs d'un objet dans l'ordre d'affichage. */
  const ordonner = (o: Record<string, unknown>) => Object.entries(o).sort(([a], [b]) => rang(a) - rang(b));
  /** Libellé d'un champ : traduit s'il est connu, sinon son nom avec des espaces. */
  function libelle(k: string) {
    return LIBELLES[k] ?? k.replaceAll("_", " ");
  }
</script>

{#if Array.isArray(v)}
  {#if v.every((x) => typeof x === "string")}
    <span class="pastilles">
      {#each v as x}{@const h = lien(String(x)) ?? (estId(String(x)) ? url.document(String(x)) : null)}{#if h}<a class="pastille" href={h}>{x}</a>{:else}<span class="pastille">{x}</span>{/if}{/each}
      {#if !v.length}<span class="discret">—</span>{/if}
    </span>
  {:else}
    <ol class="liste-objets">{#each v as x}<li><Valeur v={x} /></li>{/each}</ol>
  {/if}
{:else if estRoadmap(v)}
  <div class="horizons">
    {#each HORIZONS as [k, nom]}
      <section><h5>{nom}</h5><Valeur v={(v as Record<string, unknown>)[k]} ordre={["objectif", "recit", "besoins"]} /></section>
    {/each}
  </div>
{:else if v && typeof v === "object"}
  <dl class="champs">
    {#each ordonner(v as Record<string, unknown>) as [k, x]}
      <dt>{libelle(k)}</dt>
      <dd><Valeur v={x} cle={k} /></dd>
    {/each}
  </dl>
{:else if typeof v === "string" && estId(v) && cle !== "titre"}
  <a class="pastille" href={url.document(v)}>{v}</a>
{:else if typeof v === "string" && v.length > 0}
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  <span class="texte-lie">{@html rendre(v, lien, undefined, undefined, lien).replace(/^<p>|<\/p>\n?$/g, "")}</span>
{:else}
  <span class="texte-libre">{v === "" || v == null ? "—" : String(v)}</span>
{/if}
