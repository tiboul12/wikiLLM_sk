// Zone Projet rangée par version (EF-174, ADR-0018) : la version de chaque document est déduite de son
// frontmatter (`version`, une mineure sous sa majeure ; une ADR prend celle de sa première exigence), puis
// chaque version est mise dans l'ordre de la démarche : demande d'évolution, analyse, définition, cahier des
// charges, besoins → exigences → tests, exigences non fonctionnelles, décisions, specs, plan, autres.
//
// @req EF-174, EF-175
// @adr 0018
import type { DocumentResume } from "../types/DocumentResume";

/** Un besoin et les exigences de la version qui le couvrent ; `version` est celle du besoin (antérieure si différente). */
export type BrancheBesoin = { besoin: DocumentResume | null; id: string; exigences: { exigence: DocumentResume; tests: DocumentResume[] }[] };

/** Une version de la zone Projet, prête à afficher. */
export type Version = {
  /** Majeure : `v9`. */
  nom: string;
  /** Titre court tiré de la BDR définition ou du cahier des charges. */
  titre: string;
  /** Identifiants cités comme sources par la BDR analyse : la demande d'évolution. */
  demande: string[];
  analyse: DocumentResume[];
  definition: DocumentResume[];
  cahier: DocumentResume[];
  /** Besoins de la version, puis besoins antérieurs que la version complète. */
  besoins: BrancheBesoin[];
  /** Exigences de la version sans besoin. */
  orphelines: { exigence: DocumentResume; tests: DocumentResume[] }[];
  nonFonctionnelles: DocumentResume[];
  decisions: DocumentResume[];
  specs: DocumentResume[];
  plans: DocumentResume[];
  autres: DocumentResume[];
  /** Tous les documents de la version, pour les comptes de l'état de suivi. */
  tous: DocumentResume[];
};

/** Majeure d'une version écrite `v1.1` ou `1.1` : `v1` ; `null` si elle ne se lit pas. */
export function majeure(v: string | null | undefined): string | null {
  const m = /^v?(\d+)/i.exec((v ?? "").trim());
  return m ? `v${m[1]}` : null;
}

/** Version d'un document : la sienne, ou pour une ADR celle de sa première exigence connue. */
export function versionDe(d: DocumentResume, parId: Map<string, DocumentResume>): string | null {
  const v = majeure(d.version);
  if (v) return v;
  if (d.genre === "adr") for (const e of d.exigences) {
    const ve = majeure(parId.get(e)?.version);
    if (ve) return ve;
  }
  return null;
}

/** Genres rangés dans un emplacement précis de l'arbre ; les autres vont dans « Autres documents ». */
const PLACES = new Set(["bdr-analyse", "bdr-definition", "cahier-des-charges", "besoin", "exigence", "exigence-nf", "test", "adr", "spec", "plan"]);

/**
 * Range les documents de la zone Projet : les versions, de la plus récente à la plus ancienne, et les documents
 * transverses (sans version : backlog, ADR sans exigence…).
 */
export function rangerParVersion(documents: DocumentResume[]): { versions: Version[]; transverses: DocumentResume[] } {
  const docs = documents.filter((d) => d.zone === "projet");
  const parId = new Map(docs.map((d) => [d.id, d]));
  const parVersion = new Map<string, DocumentResume[]>();
  const transverses: DocumentResume[] = [];
  for (const d of docs) {
    const v = versionDe(d, parId);
    if (!v || d.genre === "backlog") transverses.push(d);
    else (parVersion.get(v) ?? parVersion.set(v, []).get(v)!).push(d);
  }
  const testsDe = (id: string) => docs.filter((t) => t.genre === "test" && t.exigences.includes(id));
  const versions = [...parVersion.entries()]
    .sort((a, b) => b[0].localeCompare(a[0], "fr", { numeric: true }))
    .map(([nom, ds]): Version => {
      const g = (genre: string) => ds.filter((d) => d.genre === genre);
      const exigences = g("exigence");
      const besoins: BrancheBesoin[] = g("besoin").map((b) => ({ besoin: b, id: b.id, exigences: [] }));
      const orphelines: Version["orphelines"] = [];
      for (const e of exigences) {
        const item = { exigence: e, tests: testsDe(e.id) };
        if (!e.besoins.length) {
          orphelines.push(item);
          continue;
        }
        // Une exigence qui couvre plusieurs besoins est rangée sous le premier.
        const id = e.besoins[0];
        let br = besoins.find((x) => x.id === id);
        if (!br) besoins.push((br = { besoin: parId.get(id) ?? null, id, exigences: [] }));
        br.exigences.push(item);
      }
      const analyse = g("bdr-analyse");
      const definition = g("bdr-definition");
      const cahier = g("cahier-des-charges");
      const titre = (definition[0] ?? cahier[0])?.titre.replace(/^(Définition du besoin|Cahier des charges)\s*/i, "").replace(/^v\d+\s*/, "").replace(/^\((.*)\)$/, "$1") ?? "";
      // Les tests rangés sous une exigence d'une autre version restent dans leur version : on les montre dans « Autres ».
      const testsPlaces = new Set(exigences.flatMap((e) => testsDe(e.id).map((t) => t.id)));
      return {
        nom,
        titre,
        demande: analyse.flatMap((a) => a.sources),
        analyse,
        definition,
        cahier,
        besoins,
        orphelines,
        nonFonctionnelles: g("exigence-nf"),
        decisions: g("adr"),
        specs: g("spec"),
        plans: g("plan"),
        autres: ds.filter((d) => !PLACES.has(d.genre) || (d.genre === "test" && !testsPlaces.has(d.id))),
        tous: ds,
      };
    });
  return { versions, transverses };
}

/** État de suivi d'une version (EF-175) : documents à relire, validés, et tests validés sur le total. */
export function suivi(v: Version, aRelireIds: Set<string>) {
  const tests = v.tous.filter((d) => d.genre === "test");
  return {
    documents: v.tous.length,
    aRelire: v.tous.filter((d) => aRelireIds.has(d.id)).length,
    valides: v.tous.filter((d) => ["validé", "accepté", "retenu", "fait"].includes(d.statut ?? "")).length,
    tests: tests.length,
    testsValides: tests.filter((t) => t.validation === "utilisateur" || t.validation === "modele").length,
  };
}
