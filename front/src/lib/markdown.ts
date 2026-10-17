// Rendu Markdown du wiki : liens [[...]] résolus par l'index, code coloré, Mermaid à la demande,
// HTML toujours assaini par DOMPurify (ENF-004 : un document ne peut pas exécuter de script).
//
// @req EF-042, ENF-004
import { Marked, type Tokens, type TokenizerAndRendererExtension } from "marked";
import DOMPurify from "dompurify";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import typescript from "highlight.js/lib/languages/typescript";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import ini from "highlight.js/lib/languages/ini";
import bash from "highlight.js/lib/languages/bash";
import sql from "highlight.js/lib/languages/sql";
import yaml from "highlight.js/lib/languages/yaml";
import python from "highlight.js/lib/languages/python";
import markdown from "highlight.js/lib/languages/markdown";

for (const [nom, langage] of Object.entries({ rust, typescript, javascript, json, ini, bash, sql, yaml, python, markdown }))
  hljs.registerLanguage(nom, langage);
hljs.registerAliases(["rs"], { languageName: "rust" });
hljs.registerAliases(["ts"], { languageName: "typescript" });
hljs.registerAliases(["js"], { languageName: "javascript" });
hljs.registerAliases(["toml"], { languageName: "ini" });
hljs.registerAliases(["sh", "shell", "console"], { languageName: "bash" });
hljs.registerAliases(["yml"], { languageName: "yaml" });

const echapper = (t: string) =>
  t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

/** Code coloré ; langage inconnu → texte échappé. */
export function colorer(code: string, langage = "rust"): string {
  if (langage && hljs.getLanguage(langage)) return hljs.highlight(code, { language: langage }).value;
  return echapper(code);
}

/** Identifiant d'ancre stable pour un titre. */
export function ancre(texte: string): string {
  return texte
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

/** Cible d'un lien [[brute]] : URL interne, ou null si le lien est cassé. */
export type Resolveur = (brute: string) => string | null;

type JetonLien = { type: "lienWiki"; raw: string; cible: string; libelle?: string; ancre?: string };

/**
 * Markdown → HTML assaini.
 * `refBase` : préfixe des hooks ⚓ posés sur les titres (`@wiki:projet/doc`), ou rien pour ne pas en poser.
 */
export function rendre(md: string, resoudre: Resolveur = () => null, refBase?: string): string {
  const lienWiki: TokenizerAndRendererExtension = {
    name: "lienWiki",
    level: "inline",
    start: (src: string) => src.indexOf("[["),
    tokenizer(src: string) {
      const m = /^\[\[([^\]|#]+)(#[^\]|]*)?(?:\|([^\]]+))?\]\]/.exec(src);
      if (m) return { type: "lienWiki", raw: m[0], cible: m[1].trim(), ancre: m[2], libelle: m[3] };
      return undefined;
    },
    renderer(t) {
      const j = t as unknown as JetonLien;
      const href = resoudre(j.cible);
      const texte = echapper(j.libelle ?? j.cible);
      return href
        ? `<a class="lien-wiki" href="${echapper(href + (j.ancre ?? ""))}">${texte}</a>`
        : `<span class="lien-casse" title="lien introuvable">${texte}</span>`;
    },
  };
  const m = new Marked({
    gfm: true,
    extensions: [lienWiki],
    renderer: {
      code({ text, lang }: Tokens.Code) {
        const l = (lang ?? "").split(/\s/)[0];
        if (l === "mermaid") return `<pre class="mermaid">${echapper(text)}</pre>`;
        return `<pre><code class="hljs">${colorer(text, l || "plaintext")}</code></pre>`;
      },
      heading({ tokens, depth }: Tokens.Heading) {
        const html = this.parser.parseInline(tokens);
        const id = ancre(html.replace(/<[^>]+>/g, ""));
        const hook = refBase ? `<button class="hook" data-ref="${echapper(refBase + "#" + id)}" title="Copier la référence pour un prompt">⚓</button>` : "";
        return `<h${depth} id="${id}">${html}${hook}</h${depth}>\n`;
      },
    },
  });
  const html = m.parse(md, { async: false }) as string;
  return DOMPurify.sanitize(html, { ADD_ATTR: ["data-ref"] });
}

/** Rend les blocs Mermaid d'un élément ; la bibliothèque n'est chargée que si nécessaire. */
export async function rendreMermaid(racine: HTMLElement) {
  const blocs = Array.from(racine.querySelectorAll<HTMLElement>("pre.mermaid:not([data-processed])"));
  if (!blocs.length) return;
  const mermaid = (await import("mermaid")).default;
  const sombre = matchMedia("(prefers-color-scheme: dark)").matches;
  mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme: sombre ? "dark" : "default" });
  try {
    await mermaid.run({ nodes: blocs });
  } catch {
    /* diagramme invalide : le source reste affiché */
  }
}
