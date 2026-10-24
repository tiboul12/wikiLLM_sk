<script lang="ts">
  // Section Paramètres (EF-147, EF-155, EF-156), en accordéons dans l'ordre où l'on règle : la connexion au
  // LLM de wikik (connecteur, modèle, effort, test), l'agent de développement, les prompts système (des
  // skills du projet, un par sous-accordéon, ADR-0017) et l'application reçue. Écrit dans wikik.toml et
  // `.claude/skills/` ; réservé au propriétaire (ADR-0013). `#skill-<nom>` ouvre un skill.
  import { tick } from "svelte";
  import type { Parametres } from "../types/Parametres";
  import type { Modele } from "../types/Modele";
  import type { Skill } from "../types/Skill";
  import { terminal, chargerParametres, enregistrerParametres, ecrireSkill, poster } from "../lib/terminal.svelte";
  import { reception, ecrireReception } from "../lib/reception.svelte";

  /** Copie modifiable des réglages : rien n'est écrit tant que le formulaire n'est pas enregistré. */
  let p = $state<Parametres | null>(null);
  /** Arguments du connecteur, un par ligne dans le champ ; découpés à l'envoi par `avecArguments`. */
  let arguments_ = $state("");
  /** Adresse de l'application reçue (section Réception), enregistrée à part dans le cahier. */
  let adresse = $state("");
  /** Confirmation d'enregistrement, effacée à chaque nouvel envoi. */
  let message = $state("");
  /** Erreur d'enregistrement renvoyée par le serveur. */
  let erreur = $state("");

  chargerParametres().then(async () => {
    if (!terminal.parametres) return;
    p = structuredClone($state.snapshot(terminal.parametres));
    arguments_ = p.arguments.join("\n");
    adresse = reception.etat?.url_application ?? "";
    textes = Object.fromEntries(terminal.skills.map((s) => [s.nom, s.texte]));
    chargerModeles();
    await tick();
    ouvrirAncre();
  });

  /** Description du connecteur choisi (commande par défaut, besoin d'une adresse, efforts possibles). */
  const connecteur = $derived(terminal.connecteurs.find((c) => c.id === p?.connecteur));

  // Modèles proposés par le connecteur choisi (EF-155) ; le champ reste libre.
  /** Modèles proposés par le connecteur, vides pour `cli` et `factice` qui n'en listent pas. */
  let modeles = $state<Modele[]>([]);
  /** Erreur de la liste des modèles (connecteur absent, serveur local injoignable) : le champ reste saisissable. */
  let erreurModeles = $state("");
  /** Liste des modèles en cours de chargement. */
  let chargeModeles = $state(false);
  /**
   * Demande au serveur les modèles du connecteur avec la commande et l'adresse saisies, sans les enregistrer.
   * Réservé au propriétaire : la route lance un programme local.
   */
  async function chargerModeles() {
    if (!p || !terminal.proprietaire) return;
    erreurModeles = "";
    modeles = [];
    if (["cli", "factice"].includes(p.connecteur)) return;
    chargeModeles = true;
    try {
      const q = new URLSearchParams({ connecteur: p.connecteur, commande: p.commande, url: p.url });
      const r = await fetch(`/api/parametres/modeles?${q}`);
      const v = await r.json();
      if (!r.ok) throw new Error(v.erreur ?? `${r.status}`);
      modeles = v;
    } catch (x) {
      erreurModeles = String(x instanceof Error ? x.message : x);
    } finally {
      chargeModeles = false;
    }
  }
  /**
   * Change de connecteur : modèle, effort et adresse repartent à vide, car ils n'ont pas de sens d'un connecteur à l'autre ;
   * la commande prend la valeur par défaut du nouveau connecteur (vide pour les API HTTP appelées par `curl`).
   */
  function choisirConnecteur(id: string) {
    if (!p || p.connecteur === id) return;
    p.connecteur = id;
    p.modele = "";
    p.effort = "";
    p.url = "";
    const d = terminal.connecteurs.find((c) => c.id === id);
    if (id !== "cli") p.commande = d?.commande_defaut === "curl" ? "" : (d?.commande_defaut ?? "");
    test = null;
    chargerModeles();
  }

  /** Arguments sans les lignes vides, sauf la dernière : on garde la ligne en cours de saisie. */
  const avecArguments = () => arguments_.split("\n").map((a) => a.trim()).filter((a, i, t) => a || i < t.length - 1);

  // Test de connexion : un appel minimal avec les réglages de l'écran, sans les enregistrer.
  /** Résultat du dernier test de connexion, effacé au changement de connecteur. */
  let test = $state<{ ok: boolean; fournisseur: string; reponse?: string; duree_ms?: number; erreur?: string } | null>(null);
  /** Test de connexion en cours (bouton désactivé). */
  let enTest = $state(false);
  /** Envoie les réglages affichés à `/api/parametres/tester` ; une erreur réseau est présentée comme un test raté. */
  async function tester() {
    if (!p) return;
    test = null;
    enTest = true;
    try {
      test = await poster("/api/parametres/tester", { ...p, arguments: avecArguments() });
    } catch (x) {
      test = { ok: false, fournisseur: p.connecteur, erreur: String(x instanceof Error ? x.message : x) };
    } finally {
      enTest = false;
    }
  }

  /** Enregistre les réglages dans wikik.toml, puis l'adresse de l'application si la réception est active et qu'elle a changé. */
  async function enregistrer(e: SubmitEvent) {
    e.preventDefault();
    if (!p) return;
    message = erreur = "";
    try {
      await enregistrerParametres({ ...p, arguments: avecArguments() });
      if (reception.actif && adresse.trim() !== (reception.etat?.url_application ?? "")) await ecrireReception("application", { url: adresse.trim() });
      message = "Enregistré dans wikik.toml.";
    } catch (x) {
      erreur = String(x instanceof Error ? x.message : x);
    }
  }

  // Prompts système (EF-156) : un skill par sous-accordéon, enregistré à part.
  /** Texte de chaque skill tel qu'édité, par nom ; initialisé depuis le projet au chargement. */
  let textes = $state<Record<string, string>>({});
  /** Message par skill après enregistrement, rétablissement ou installation. */
  let etatSkill = $state<Record<string, string>>({});
  /** Groupes de skills affichés : identifiant, titre, à qui ils s'adressent (ADR-0017). */
  const GROUPES: [string, string, string][] = [
    ["agent", "Pour l'agent de développement", "Ce que l'agent qui code dans le projet (Claude Code, dans le terminal) reçoit ou trouve dans .claude/skills/."],
    ["llm", "Pour le LLM de wikik", "Ce que wikik envoie à son LLM (Design thinking, réception, recherche) et à tout client de son serveur MCP."],
    ["structure", "Structure des documents", "Comment sont rédigés les documents de chaque section du projet : donnés au LLM quand il en produit, et à l'agent."],
  ];
  /** Skills d'un groupe, dans l'ordre fourni par le serveur. */
  const deGroupe = (g: string) => terminal.skills.filter((s) => s.groupe === g);
  /** Écrit le skill dans son `SKILL.md` du projet. */
  async function enregistrerSkill(s: Skill) {
    etatSkill[s.nom] = "";
    try {
      await ecrireSkill(s.nom, textes[s.nom]);
      etatSkill[s.nom] = "Enregistré dans " + s.chemin;
    } catch (x) {
      etatSkill[s.nom] = "Erreur : " + String(x instanceof Error ? x.message : x);
    }
  }
  /** Rétablit le texte livré avec wikik en supprimant le fichier du projet, après confirmation. */
  async function retablirSkill(s: Skill) {
    if (!confirm(`Revenir au texte d'origine de « ${s.titre} » ? Le fichier ${s.chemin} sera supprimé.`)) return;
    await ecrireSkill(s.nom, null);
    textes[s.nom] = terminal.skills.find((x) => x.nom === s.nom)?.texte ?? "";
    etatSkill[s.nom] = "Texte d'origine rétabli.";
  }
  /** Installe le skill dans `.claude/skills/` pour que l'agent de développement le trouve. */
  async function installerSkill(s: Skill) {
    await ecrireSkill(s.nom, textes[s.nom]);
    etatSkill[s.nom] = "Installé dans " + s.chemin + " : l'agent le trouve.";
  }

  /** `#skill-<nom>` (lien ⚙ d'une section du projet) : ouvre ce skill et le montre. */
  function ouvrirAncre() {
    const id = decodeURIComponent(location.hash.slice(1));
    if (!id) return;
    const el = document.getElementById(id) as HTMLDetailsElement | null;
    if (!el) return;
    for (let d: HTMLElement | null = el; d; d = d.parentElement?.closest("details") ?? null) (d as HTMLDetailsElement).open = true;
    requestAnimationFrame(() => el.scrollIntoView({ block: "start" }));
  }
</script>

<h1>Paramètres</h1>
<p class="discret">Réglages du projet : la connexion au LLM et l'agent sont écrits dans <code>wikik.toml</code> (une valeur laissée par défaut n'y est pas écrite), les prompts système sont des skills dans <code>.claude/skills/</code>. Le LLM change sans relancer wikik.</p>
{#if !p}
  <p class="discret">Chargement…</p>
{:else}
  {#if !terminal.proprietaire}
    <p class="alerte-acces">Lecture seule : modifier ces réglages (et ouvrir le terminal) est réservé au propriétaire. Ouvre wikik depuis sa machine, ou avec l'adresse à clé affichée au lancement de <code>wikik</code>.</p>
  {/if}
  <form class="formulaire parametres" onsubmit={enregistrer}>
    <fieldset disabled={!terminal.proprietaire}>
      <details class="bloc-parametres" open id="connexion">
        <summary><span class="num">1</span> Connexion au LLM <small>{connecteur?.nom ?? p.connecteur}{p.modele ? ` · ${p.modele}` : ""}{p.effort ? ` · effort ${p.effort}` : ""}</small></summary>
        <p class="discret">Le LLM que wikik appelle pour le Design thinking, la réception (écrans manquants) et la recherche assistée. L'authentification reste celle du CLI ou du serveur local : wikik ne garde aucune clé.</p>
        {#if terminal.factice}<p class="discret">wikik tourne avec <code>--factice</code> : ces réglages serviront au prochain lancement sans.</p>{/if}
        <div class="connecteurs" role="radiogroup" aria-label="Fournisseur">
          {#each terminal.connecteurs as c (c.id)}
            <button type="button" class="connecteur" class:choisi={p.connecteur === c.id} role="radio" aria-checked={p.connecteur === c.id} onclick={() => choisirConnecteur(c.id)}>
              <strong>{c.nom}</strong>
              <small>{c.aide}</small>
              {#if c.mcp}<span class="pastille">lit le projet par MCP</span>{/if}
            </button>
          {/each}
        </div>

        {#if connecteur && !["cli", "factice"].includes(p.connecteur)}
          <div class="ligne-champs">
            <label class="champ-large"><span>Modèle</span>
              <input list="modeles-proposes" bind:value={p.modele} placeholder={modeles.length ? "par défaut du fournisseur" : "nom du modèle (vide : par défaut)"} />
              <datalist id="modeles-proposes">{#each modeles as m (m.id)}<option value={m.id}>{m.libelle}</option>{/each}</datalist>
              <small>
                {#if chargeModeles}Liste des modèles…{:else if erreurModeles}Liste indisponible : {erreurModeles}{:else if modeles.length}{modeles.length} modèle(s) proposé(s) : {modeles.map((m) => m.id).join(", ")}{:else}Aucun modèle proposé : écris son nom.{/if}
                <button type="button" class="discret-bouton" onclick={chargerModeles}>actualiser</button>
              </small>
            </label>
            {#if connecteur.efforts.length}
              <label><span>Effort</span>
                <select bind:value={p.effort}>
                  <option value="">par défaut</option>
                  {#each connecteur.efforts as e}<option value={e}>{e}</option>{/each}
                </select>
              </label>
            {/if}
          </div>
          {#if connecteur.serveur}
            <label><span>Adresse du serveur</span><input bind:value={p.url} placeholder={connecteur.url_defaut} onchange={chargerModeles} /><small>Vide : {connecteur.url_defaut} (celle de Shelley Administration si elle est installée sur cette machine).</small></label>
          {:else}
            <label><span>Commande</span><input bind:value={p.commande} placeholder={connecteur.commande_defaut} /><small>Le binaire du CLI, s'il n'est pas dans le PATH sous ce nom.</small></label>
          {/if}
        {/if}
        {#if p.connecteur === "cli"}
          <label><span>Commande</span><input bind:value={p.commande} /></label>
          <label><span>Arguments (un par ligne)</span><textarea rows={8} class="mono" bind:value={arguments_}></textarea><small>Jetons remplacés à chaque appel : <code>{"{consigne}"}</code>, <code>{"{schema}"}</code>, <code>{"{mcp}"}</code>. La sortie doit être un flux <code>stream-json</code> de Claude Code.</small></label>
        {/if}
        {#if p.connecteur !== "factice"}
          <label class="champ-court"><span>Délai maximum (s)</span><input type="number" min="10" bind:value={p.delai} /></label>
        {/if}
        <div class="actions">
          <button type="button" onclick={tester} disabled={enTest}>{enTest ? "Test en cours…" : "Tester la connexion"}</button>
          {#if test}
            {#if test.ok}<span class="succes">✓ {test.fournisseur} répond en {((test.duree_ms ?? 0) / 1000).toFixed(1)} s : « {test.reponse} »</span>
            {:else}<span class="erreur">✗ {test.fournisseur} : {test.erreur}</span>{/if}
          {/if}
        </div>
      </details>

      <details class="bloc-parametres" id="agent">
        <summary><span class="num">2</span> Agent de développement <small>{p.agent}</small></summary>
        <label><span>Commande de l'agent</span><input bind:value={p.agent} placeholder="claude" /><small>Lancée dans le terminal intégré par « Lancer l'agent » et par « Traiter avec l'agent », qui lui envoie le skill <code>wikik-traiter-reception</code>.</small></label>
      </details>

      <details class="bloc-parametres" id="application">
        <summary><span class="num">3</span> Application reçue <small>{adresse || "déduite"}</small></summary>
        <label><span>Adresse de l'application</span><input bind:value={adresse} placeholder={reception.etat?.url_detectee === "/" ? "ce serveur (déduit)" : (reception.etat?.url_detectee ?? "http://10.0.0.21:4200")} /><small>Vide : déduite du projet ({reception.etat?.url_detectee === "/" ? "ce serveur" : (reception.etat?.url_detectee ?? "rien de trouvé")}). Sert aux liens « Ouvrir cette page » du cahier de réception.</small></label>
      </details>

      <div class="actions">
        <button class="principal" type="submit">Enregistrer</button>
        {#if message}<span class="succes">{message}</span>{/if}
      </div>
      {#if erreur}<p class="erreur">{erreur}</p>{/if}
    </fieldset>
  </form>

  <details class="bloc-parametres" id="prompts">
    <summary><span class="num">4</span> Prompts système <small>{terminal.skills.length} skills · {terminal.skills.filter((s) => s.modifie).length} modifié(s)</small></summary>
    <p class="discret">Chaque consigne donnée à un LLM est un skill au format Claude Code (<code>SKILL.md</code> : un frontmatter <code>name</code> et <code>description</code>, puis des sections). Ce sont les textes réellement envoyés. Tant qu'un skill n'est pas dans le projet, son texte d'origine sert ; l'enregistrer l'écrit dans <code>.claude/skills/</code>, où l'agent de développement le trouve aussi.</p>
    {#each GROUPES as [g, titre, aide] (g)}
      <h3>{titre}</h3>
      <p class="discret">{aide}</p>
      {#each deGroupe(g) as s (s.nom)}
        <details class="skill" id="skill-{s.nom}">
          <summary>
            <strong>{s.titre}</strong> <code>{s.nom}</code>
            {#if s.modifie}<span class="pastille statut-proposé">modifié</span>{:else if s.installe}<span class="pastille">dans le projet</span>{:else}<span class="pastille discret">texte d'origine</span>{/if}
          </summary>
          <p>{s.role}</p>
          <fieldset disabled={!terminal.proprietaire}>
            <textarea class="mono texte-skill" rows={16} bind:value={textes[s.nom]} aria-label="Texte du skill {s.nom}"></textarea>
            <div class="actions">
              <button class="principal" type="button" onclick={() => enregistrerSkill(s)} disabled={textes[s.nom] === s.texte && s.installe}>Enregistrer le skill</button>
              {#if !s.installe && textes[s.nom] === s.texte}<button type="button" onclick={() => installerSkill(s)}>Installer dans le projet</button>{/if}
              {#if s.installe}<button type="button" class="discret-bouton" onclick={() => retablirSkill(s)}>Revenir au texte d'origine</button>{/if}
              <small class="discret">{s.chemin}</small>
              {#if etatSkill[s.nom]}<span class={etatSkill[s.nom].startsWith("Erreur") ? "erreur" : "succes"}>{etatSkill[s.nom]}</span>{/if}
            </div>
          </fieldset>
        </details>
      {/each}
    {/each}
  </details>
{/if}

<style>
  .bloc-parametres {
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    padding: 4px 16px 12px;
    margin: 0 0 12px;
    background: var(--fond);
  }
  .bloc-parametres > summary {
    cursor: pointer;
    padding: 10px 0;
    font-weight: 650;
    font-size: 16px;
  }
  .bloc-parametres > summary small {
    font-weight: 400;
    color: var(--texte-2);
    margin-left: 8px;
  }
  .num {
    display: inline-grid;
    place-items: center;
    width: 22px;
    height: 22px;
    margin-right: 6px;
    border-radius: 50%;
    background: var(--accent-fond);
    color: var(--accent);
    font-size: 12.5px;
  }
  .connecteurs {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 8px;
    margin: 8px 0 14px;
  }
  .connecteur {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    text-align: left;
    padding: 10px 12px;
    border: 1px solid var(--bord);
    border-radius: var(--rayon);
    background: var(--fond-2);
    color: var(--texte);
    font: inherit;
    cursor: pointer;
  }
  .connecteur small {
    color: var(--texte-2);
    font-size: 12.5px;
    line-height: 1.35;
  }
  .connecteur.choisi {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
    background: var(--accent-fond);
  }
  .ligne-champs {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }
  .champ-large {
    flex: 1 1 18rem;
  }
  .champ-court input {
    max-width: 8rem;
  }
  .skill {
    border-top: 1px solid var(--bord);
    padding: 6px 0;
  }
  .skill > summary {
    cursor: pointer;
    padding: 6px 0;
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .skill fieldset {
    border: none;
    padding: 0;
    margin: 0;
  }
  .texte-skill {
    width: 100%;
    font-size: 12.5px;
    line-height: 1.45;
  }
  h3 {
    margin: 18px 0 2px;
    font-size: 15px;
  }
</style>
