<script lang="ts">
  // Correction d'une proposition avant de la garder : un champ par propriété du schéma de la fiche.
  type Schema = { type?: string; enum?: string[]; properties?: Record<string, Schema>; items?: Schema; required?: string[] };

  let {
    schema,
    valeur,
    valider,
    annuler,
  }: { schema: Schema; valeur: Record<string, unknown>; valider: (v: Record<string, unknown>) => void; annuler: () => void } = $props();

  // svelte-ignore state_referenced_locally
  const proprietes = Object.entries(schema.properties ?? {});
  // Brouillon : textes tels quels, listes de textes séparées par des virgules, le reste en JSON.
  // svelte-ignore state_referenced_locally
  let brouillon = $state<Record<string, string>>(
    Object.fromEntries(
      proprietes.map(([k, s]) => {
        const v = valeur[k];
        if (s.type === "array" && s.items?.type === "string") return [k, Array.isArray(v) ? v.join(", ") : ""];
        if (s.type === "string" || s.type === "integer") return [k, v == null ? "" : String(v)];
        return [k, JSON.stringify(v ?? null, null, 2)];
      }),
    ),
  );
  let erreur = $state("");

  function envoyer(e: SubmitEvent) {
    e.preventDefault();
    const sortie: Record<string, unknown> = { ...valeur };
    try {
      for (const [k, s] of proprietes) {
        const t = brouillon[k];
        if (s.type === "array" && s.items?.type === "string") sortie[k] = t.split(",").map((x) => x.trim()).filter(Boolean);
        else if (s.type === "integer") sortie[k] = Number.parseInt(t, 10);
        else if (s.type === "string") sortie[k] = t;
        else sortie[k] = JSON.parse(t);
      }
    } catch (x) {
      erreur = `JSON invalide : ${x}`;
      return;
    }
    valider(sortie);
  }
  const lignes = (t: string) => Math.min(12, Math.max(1, Math.ceil(t.length / 70) + (t.match(/\n/g)?.length ?? 0)));
</script>

<form class="formulaire" onsubmit={envoyer}>
  {#each proprietes as [k, s]}
    <label>
      <span>{k.replaceAll("_", " ")}{schema.required?.includes(k) ? " *" : ""}</span>
      {#if s.enum}
        <select bind:value={brouillon[k]}>{#each s.enum as o}<option value={o}>{o}</option>{/each}</select>
      {:else if s.type === "string"}
        <textarea rows={lignes(brouillon[k])} bind:value={brouillon[k]}></textarea>
      {:else if s.type === "integer"}
        <input type="number" bind:value={brouillon[k]} />
      {:else}
        <textarea class="mono" rows={8} bind:value={brouillon[k]}></textarea>
      {/if}
    </label>
  {/each}
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
  <div class="actions">
    <button class="principal" type="submit">Garder corrigé</button>
    <button type="button" onclick={annuler}>Annuler</button>
  </div>
</form>
