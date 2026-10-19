<script lang="ts">
  // Icônes de la barre d'activité, en SVG (aucune police ni ressource externe).
  // @relu 2026-10-19 80333182
  let { nom, taille = 22 }: { nom: string; taille?: number } = $props();
  const CHEMINS: Record<string, string> = {
    accueil: "M3 10.5 12 3l9 7.5M5 9v11h5v-6h4v6h5V9",
    produit: "M8 6 2 12l6 6M16 6l6 6-6 6M14 4l-4 16",
    projet: "M7 3h7l5 5v13H7zM14 3v5h5M10 13h6M10 17h6",
    atelier: "M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9c.6.5 1 1.2 1 2V17h5v-1.1c0-.8.4-1.5 1-2A6 6 0 0 0 12 3z",
    reception: "M9 4h6v3H9zM7 5H5v16h14V5h-2M8.5 13.5l2.5 2.5 4.5-5",
    parametres: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
    terminal: "M4 17l6-6-6-6M12 19h8",
    graphe: "M6 6m-2.5 0a2.5 2.5 0 1 0 5 0a2.5 2.5 0 1 0-5 0M18 8m-2.5 0a2.5 2.5 0 1 0 5 0a2.5 2.5 0 1 0-5 0M10 18m-2.5 0a2.5 2.5 0 1 0 5 0a2.5 2.5 0 1 0-5 0M8.3 7.2l7.4 1M7 8.3l2.2 7.4M16.3 10l-4.6 6",
  };
</script>

<svg width={taille} height={taille} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d={CHEMINS[nom] ?? ""} />
</svg>
