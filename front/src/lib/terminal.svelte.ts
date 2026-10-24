// Terminal intégré et paramètres (EF-147, EF-148) : volet droit, droits du navigateur (propriétaire ou
// non, ADR-0013), envoi de la consigne de traitement à l'agent, arrêt du serveur (ADR-0014).
//
// @req EF-147, EF-148
import type { Parametres } from "../types/Parametres";
import type { DescriptionConnecteur } from "../types/DescriptionConnecteur";
import type { Skill } from "../types/Skill";
import { statique } from "./statique";

/**
 * `ouvert` : volet affiché ; `proprietaire` : ce navigateur peut ouvrir le terminal et régler l'IA ;
 * `arrete` : le serveur a été arrêté ; `connecteurs` et `skills` : ce que montrent les Paramètres
 * (EF-155, EF-156).
 */
export const terminal = $state({
  ouvert: false,
  proprietaire: false,
  factice: false,
  arrete: false,
  parametres: null as Parametres | null,
  connecteurs: [] as DescriptionConnecteur[],
  skills: [] as Skill[],
});

/** Charge les paramètres et les droits du navigateur ; sans effet dans le site exporté.
 * @relu 2026-10-22 4ffabc6c
 */
export async function chargerParametres() {
  if (statique) return;
  const r = await fetch("/api/parametres");
  if (!r.ok) return;
  const v = await r.json();
  terminal.parametres = v.parametres;
  terminal.proprietaire = v.proprietaire;
  terminal.factice = v.factice;
  terminal.connecteurs = v.connecteurs ?? [];
  terminal.skills = v.skills ?? [];
}

/** POST JSON vers une route du serveur ; l'erreur du serveur est levée. */
export async function poster<T>(route: string, corps: unknown): Promise<T> {
  const r = await fetch(route, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(corps) });
  const v = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(v.erreur ?? `${r.status}`);
  return v as T;
}

/** Arrête le serveur du projet (ADR-0014) : il faudra relancer `wikik` dans le projet. */
export async function arreterServeur() {
  await poster("/api/serveur/arreter", {});
  terminal.ouvert = false;
  terminal.arrete = true;
}

/** Enregistre les paramètres dans wikik.toml ; le LLM change sans relancer wikik. */
export async function enregistrerParametres(p: Parametres) {
  await poster("/api/parametres", p);
  await chargerParametres();
}

/** Écrit un skill modifié dans le projet (`.claude/skills/<nom>/SKILL.md`), ou revient à son origine. */
export async function ecrireSkill(nom: string, texte: string | null) {
  terminal.skills = texte === null ? await poster<Skill[]>("/api/skills/retablir", { nom }) : await poster<Skill[]>("/api/skills", { nom, texte });
}

/**
 * Envoie à l'agent la consigne de traitement de la réception (EF-156) : le skill est d'abord installé
 * dans le projet ; un agent Claude Code le reçoit par son nom (`/wikik-traiter-reception`), un autre agent
 * en reçoit le texte.
 * @relu 2026-10-24 e63544ec
 */
export async function traiterAvecAgent(): Promise<string> {
  return lancerSkill("wikik-traiter-reception");
}

/**
 * Envoie un skill à l'agent du terminal : il est d'abord installé dans le projet ; un agent Claude Code le reçoit
 * par son nom (`/<nom>`), un autre agent en reçoit le texte.
 */
export async function lancerSkill(nom: string): Promise<string> {
  const corps = await poster<string>("/api/skills/installer", { nom });
  const agent = (terminal.parametres?.agent ?? "claude").trim().split(/\s+/)[0].split("/").pop() ?? "";
  return envoyer(agent.startsWith("claude") ? `/${nom}` : corps, "consigne");
}

/**
 * Envoie un texte au terminal : `consigne` (pour l'agent : collée s'il tourne, sinon l'agent est lancé
 * avec elle) ou `commande` (tapée telle quelle). Ouvre le volet et attend que le terminal existe.
 */
export async function envoyer(texte: string, mode: "consigne" | "commande"): Promise<string> {
  terminal.ouvert = true;
  for (let i = 0; ; i++) {
    try {
      return await poster<string>("/api/terminal/envoyer", { texte, mode });
    } catch (e) {
      // Le volet vient de s'ouvrir : le terminal se crée à sa connexion.
      if (i >= 20 || !String(e).includes("ouvre d'abord")) throw e;
      await new Promise((r) => setTimeout(r, 150));
    }
  }
}
