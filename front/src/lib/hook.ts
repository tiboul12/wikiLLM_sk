// Hooks ⚓ : copient une référence `@wiki:` à coller dans un prompt (résolue par `wikik query ref`).
//
// @req EF-045
/** Nom du projet servi, renseigné au chargement : préfixe des références `@wiki:`. */
export const projetCourant = { nom: "" };

/** Référence `@wiki:<projet>/<cible>` d'un symbole ou d'un document. */
export function reference(cible: string): string {
  return `@wiki:${projetCourant.nom}/${cible}`;
}

/**
 * Copie dans le presse-papiers ; `false` si le navigateur refuse. Hors HTTPS et hors `localhost` (wikik
 * ouvert depuis un autre poste, REC-020), `navigator.clipboard` n'existe pas : on passe par une zone de
 * texte et `execCommand("copy")`, encore accepté dans ce cas.
 */
export async function copier(texte: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(texte);
    return true;
  } catch {
    try {
      const zone = document.createElement("textarea");
      zone.value = texte;
      zone.setAttribute("readonly", "");
      zone.style.cssText = "position: fixed; top: 0; left: 0; opacity: 0";
      document.body.appendChild(zone);
      zone.select();
      const ok = document.execCommand("copy");
      zone.remove();
      return ok;
    } catch {
      return false;
    }
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
