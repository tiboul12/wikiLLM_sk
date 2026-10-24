<script lang="ts">
  // Arbre des versions de la zone Projet (EF-174, ADR-0018), partagé par le volet et l'accueil Projet : pour
  // chaque version, la demande d'évolution, l'analyse, la définition, le cahier des charges, les besoins avec
  // leurs exigences et leurs tests, puis exigences non fonctionnelles, décisions, specs, plan et autres. Chaque
  // nœud se replie (fermé à l'arrivée, sauf `ouverte` pour la version la plus récente) ; un point orange marque
  // ce qui attend une relecture.
  // @req EF-174
  import type { DocumentResume } from "../types/DocumentResume";
  import { url, estCourant, routeur } from "../lib/routeur.svelte";
  import type { Version } from "../lib/versions";

  let { versions, aRelireIds, ouverte = false }: { versions: Version[]; aRelireIds: Set<string>; ouverte?: boolean } = $props();
  /** Lien de la page affichée ? Lire la route rend le résultat réactif. */
  const actif = (href: string) => (routeur.route, estCourant(href));
  /** Marque d'un test : validé, en échec ou à valider. */
  const marqueTest = (t: DocumentResume) => (t.validation === "utilisateur" || t.validation === "modele" ? "✓" : t.validation === "echec" ? "✗" : "…");
  /** Documents à relire parmi une liste, pour le compteur d'un nœud. */
  const attente = (ds: DocumentResume[]) => ds.filter((d) => aRelireIds.has(d.id)).length;
</script>

{#snippet doc(d: DocumentResume, libelle?: string)}
  {@const href = url.document(d.id)}
  <a {href} class:actif={actif(href)} title={d.titre}>
    <span class="texte">{#if libelle}<span class="etape">{libelle}</span>{/if}<span class="id">{d.id}</span> {d.titre === d.id || libelle ? "" : d.titre}</span>
    {#if aRelireIds.has(d.id)}<span class="compteur faible" title="à relire">●</span>{/if}
  </a>
{/snippet}

{#snippet exigence(e: DocumentResume, tests: DocumentResume[])}
  <li>
    {#if tests.length}
      <details class="noeud">
        <summary>{@render doc(e)}</summary>
        <ul>{#each tests as t (t.id)}{@const ht = url.document(t.id)}<li><a href={ht} class:actif={actif(ht)} title={t.titre}><span class="texte"><span class="id">{t.id}</span> {t.titre}</span><span class="compteur" class:faible={marqueTest(t) !== "✓"}>{marqueTest(t)}</span></a></li>{/each}</ul>
      </details>
    {:else}
      {@render doc(e)}
    {/if}
  </li>
{/snippet}

{#snippet groupe(nom: string, ds: DocumentResume[])}
  {#if ds.length}
    <li>
      <details class="noeud">
        <summary><span class="nom-groupe">{nom} <small>{ds.length}</small>{#if attente(ds)}<span class="compteur faible">{attente(ds)}</span>{/if}</span></summary>
        <ul>{#each ds as d (d.id)}<li>{@render doc(d)}</li>{/each}</ul>
      </details>
    </li>
  {/if}
{/snippet}

<ul class="arbre-versions">
  {#each versions as v, i (v.nom)}
    {@const n = attente(v.tous)}
    <li>
      <details class="noeud version" open={ouverte && i === 0}>
        <summary>
          <span class="nom-groupe"><strong>{v.nom}</strong>{#if v.titre}<span class="titre-version">{v.titre}</span>{/if}<small>{v.tous.length}</small>{#if n}<span class="compteur faible" title="{n} à relire">{n}</span>{/if}</span>
        </summary>
        <ul>
          {#if v.demande.length}
            <li>
              <details class="noeud">
                <summary><span class="nom-groupe"><span class="etape">Demande</span> d'évolution <small>{v.demande.length}</small></span></summary>
                <ul class="demande">{#each v.demande as id (id)}<li><a href={url.document(id)} class:actif={actif(url.document(id))}><span class="id">{id}</span></a></li>{/each}</ul>
              </details>
            </li>
          {/if}
          {#each v.analyse as d (d.id)}<li>{@render doc(d, "Analyse")}</li>{/each}
          {#each v.definition as d (d.id)}<li>{@render doc(d, "Définition")}</li>{/each}
          {#each v.cahier as d (d.id)}
            <li>
              <details class="noeud">
                <summary>{@render doc(d, "Cahier des charges")}</summary>
                <ul>
                  {#each v.besoins as b (b.id)}
                    <li>
                      <details class="noeud">
                        <summary>
                          {#if b.besoin}{@render doc(b.besoin)}{:else}<span class="id">{b.id}</span>{/if}
                          {#if b.besoin && b.besoin.version && !b.besoin.version.startsWith(v.nom)}<span class="pastille petite" title="besoin d'une version antérieure">{b.besoin.version}</span>{/if}
                        </summary>
                        {#if b.exigences.length}
                          <ul>{#each b.exigences as x (x.exigence.id)}{@render exigence(x.exigence, x.tests)}{/each}</ul>
                        {:else}
                          <p class="discret vide">Aucune exigence.</p>
                        {/if}
                      </details>
                    </li>
                  {/each}
                  {#each v.orphelines as x (x.exigence.id)}{@render exigence(x.exigence, x.tests)}{/each}
                </ul>
              </details>
            </li>
          {:else}
            {#if v.besoins.length || v.orphelines.length}
              <li>
                <details class="noeud">
                  <summary><span class="nom-groupe"><span class="etape">Besoins</span> <small>{v.besoins.length}</small></span></summary>
                  <ul>
                    {#each v.besoins as b (b.id)}
                      <li>
                        <details class="noeud">
                          <summary>{#if b.besoin}{@render doc(b.besoin)}{:else}<span class="id">{b.id}</span>{/if}</summary>
                          <ul>{#each b.exigences as x (x.exigence.id)}{@render exigence(x.exigence, x.tests)}{/each}</ul>
                        </details>
                      </li>
                    {/each}
                    {#each v.orphelines as x (x.exigence.id)}{@render exigence(x.exigence, x.tests)}{/each}
                  </ul>
                </details>
              </li>
            {/if}
          {/each}
          {@render groupe("Exigences non fonctionnelles", v.nonFonctionnelles)}
          {@render groupe("Décisions (ADR)", v.decisions)}
          {@render groupe("Specs", v.specs)}
          {#each v.plans as d (d.id)}<li>{@render doc(d, "Plan")}</li>{/each}
          {@render groupe("Autres documents", v.autres)}
        </ul>
      </details>
    </li>
  {/each}
</ul>

<style>
  .arbre-versions {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .arbre-versions :global(ul) {
    list-style: none;
    margin: 0 0 0 10px;
    padding: 0 0 0 6px;
    border-left: 1px solid var(--bord);
  }
  .arbre-versions :global(li a) {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
    padding: 3px 6px;
    border-radius: 5px;
    color: var(--texte);
    font-size: 13.5px;
    white-space: nowrap;
  }
  .arbre-versions :global(.texte) {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .arbre-versions :global(.compteur) {
    float: none;
    margin-left: auto;
    flex: none;
    font-size: 11.5px;
    color: var(--texte-2);
  }
  .arbre-versions :global(.compteur.faible) {
    color: var(--orange);
  }
  .arbre-versions :global(.id) {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--texte-2);
  }
  .arbre-versions :global(li a:hover) {
    background: var(--fond-3);
    text-decoration: none;
  }
  .arbre-versions :global(li a.actif) {
    background: var(--accent-fond);
    color: var(--accent);
    font-weight: 550;
  }
  .nom-groupe {
    flex: 1;
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
    padding: 3px 6px;
    font-size: 13.5px;
    color: var(--texte);
  }
  .nom-groupe small {
    color: var(--texte-2);
  }
  .version > summary .nom-groupe {
    font-size: 14px;
  }
  .titre-version {
    color: var(--texte-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .etape {
    color: var(--accent);
    font-weight: 600;
    margin-right: 6px;
  }
  .petite {
    font-size: 11px;
    padding: 0 6px;
  }
  .vide {
    margin: 2px 0 4px 24px;
    font-size: 12.5px;
  }
  .demande {
    display: flex;
    flex-wrap: wrap;
  }
  .demande :global(a) {
    padding: 1px 5px;
  }
</style>
