<script lang="ts">
  // Volet droit du terminal (EF-148) : le shell du serveur, ouvert dans le dossier du projet, affiché par
  // xterm.js et relié par WebSocket ; largeur réglable en tirant son bord. Fermer le volet ne ferme pas
  // le terminal : l'agent continue, sa sortie est rejouée à la réouverture.
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import { terminal, envoyer } from "../lib/terminal.svelte";

  let { largeur = $bindable(560) }: { largeur?: number } = $props();
  /** Élément où xterm.js dessine le terminal. */
  let hote: HTMLElement;
  /** État de la liaison WebSocket, affiché dans l'en-tête du volet. */
  let etat = $state<"connexion" | "ouvert" | "ferme">("connexion");
  /** Erreur du lancement de l'agent. */
  let erreur = $state("");
  /** Liaison avec le shell du serveur (`/api/terminal`). */
  let ws: WebSocket | null = null;
  /** Terminal xterm.js, créé au montage. */
  let term: Terminal | null = null;
  /** Module qui ajuste colonnes et lignes à la taille du volet. */
  let ajuster: FitAddon | null = null;
  /** Suit la taille du volet pour réajuster le terminal. */
  let observateur: ResizeObserver | null = null;

  /** Ajuste le terminal au volet et transmet colonnes et lignes au shell, sinon les programmes plein écran se dessinent mal. */
  function taille() {
    if (!term || !ajuster) return;
    ajuster.fit();
    if (ws?.readyState === WebSocket.OPEN) ws.send(JSON.stringify({ cols: term.cols, rows: term.rows }));
  }

  /** Ouvre la liaison WebSocket (wss sous HTTPS) ; à l'ouverture, le serveur rejoue la sortie gardée du shell. */
  function connecter() {
    erreur = "";
    etat = "connexion";
    ws = new WebSocket(`${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/api/terminal`);
    ws.binaryType = "arraybuffer";
    ws.onopen = () => {
      etat = "ouvert";
      taille();
      term?.focus();
    };
    ws.onmessage = (m) => term?.write(new Uint8Array(m.data as ArrayBuffer));
    ws.onclose = () => (etat = "ferme");
  }

  onMount(() => {
    const sombre = matchMedia("(prefers-color-scheme: dark)").matches;
    term = new Terminal({
      fontFamily: getComputedStyle(document.documentElement).getPropertyValue("--mono"),
      fontSize: 13,
      cursorBlink: true,
      theme: sombre ? { background: "#14161a" } : { background: "#ffffff", foreground: "#1d2026", cursor: "#1d2026", selectionBackground: "#c4d6fb" },
    });
    ajuster = new FitAddon();
    term.loadAddon(ajuster);
    term.open(hote);
    const encodeur = new TextEncoder();
    term.onData((d) => ws?.readyState === WebSocket.OPEN && ws.send(encodeur.encode(d)));
    observateur = new ResizeObserver(() => taille());
    observateur.observe(hote);
    connecter();
  });
  onDestroy(() => {
    observateur?.disconnect();
    ws?.close();
    term?.dispose();
  });

  /** Tape la commande de l'agent configurée (`claude` par défaut) dans le terminal. */
  async function lancerAgent() {
    erreur = "";
    try {
      await envoyer(terminal.parametres?.agent ?? "claude", "commande");
      term?.focus();
    } catch (e) {
      erreur = String(e instanceof Error ? e.message : e);
    }
  }

  /** Tire le bord gauche du volet : largeur bornée (320 px au moins, 300 px laissés à la page), enregistrée au relâchement. */
  function tirer(e: PointerEvent) {
    const p = e.currentTarget as HTMLElement;
    p.setPointerCapture(e.pointerId);
    const bouger = (m: PointerEvent) => (largeur = Math.min(innerWidth - 300, Math.max(320, innerWidth - m.clientX)));
    const lacher = () => {
      p.removeEventListener("pointermove", bouger);
      try {
        localStorage.setItem("wikik.terminal", String(largeur));
      } catch {
        /* sans stockage, la largeur vaut pour la session */
      }
    };
    p.addEventListener("pointermove", bouger);
    p.addEventListener("pointerup", lacher, { once: true });
  }
</script>

<aside class="panneau-terminal" style="width: {largeur}px" aria-label="Terminal">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="poignee-terminal" onpointerdown={tirer} title="Tirer pour élargir le terminal"></div>
  <header>
    <strong>Terminal</strong>
    <span class="discret">{etat === "ouvert" ? "dossier du projet" : etat === "connexion" ? "connexion…" : "déconnecté"}</span>
    <span class="espace"></span>
    {#if etat === "ferme"}<button onclick={connecter}>Reconnecter</button>{/if}
    <button onclick={lancerAgent} title="Tape « {terminal.parametres?.agent ?? 'claude'} » dans le terminal">Lancer l'agent</button>
    <button class="fermer" onclick={() => (terminal.ouvert = false)} aria-label="Fermer le volet" title="Fermer le volet (le terminal continue)">×</button>
  </header>
  {#if erreur}<p class="erreur">{erreur}</p>{/if}
  <div class="hote-terminal" bind:this={hote}></div>
</aside>
