// Hooks ⚓ : copient une référence `@wiki:` à coller dans un prompt (résolue par `wikik query ref`).
export const projetCourant = { nom: "" };

export function reference(cible: string): string {
  return `@wiki:${projetCourant.nom}/${cible}`;
}

export async function copier(texte: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(texte);
    return true;
  } catch {
    return false;
  }
}

/** Délégation : tout bouton `.hook[data-ref]` d'un contenu rendu copie sa référence. */
export function brancherHooks(racine: HTMLElement) {
  racine.addEventListener("click", async (e) => {
    const b = (e.target as Element).closest<HTMLButtonElement>("button.hook[data-ref]");
    if (!b) return;
    e.preventDefault();
    const ok = await copier(b.dataset.ref!);
    b.classList.add(ok ? "copie" : "echec");
    setTimeout(() => b.classList.remove("copie", "echec"), 1200);
  });
}
