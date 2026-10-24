---
id: BDR-v9-2
type: bdr-definition
titre: "Définition du besoin v9 (retours de la réception de la v8)"
statut: proposé
version: v9
sources: [BDR-v9-1]
---

# BDR v9, partie 2 : définition du besoin

## Objectifs mesurables

- Les 38 remarques ouvertes sont traitées, chacune avec son commit, dans le cahier de réception.
- Tibo voit une version entière (de la demande aux tests) en un seul arbre, et ce qui attend sa décision.
- Tibo atteint n'importe quel document depuis son téléphone sans passer par le volet.
- Les consignes d'un agent se règlent sur la page où il travaille.

## Périmètre (inclus / exclu)

**Inclus** : onglets, navigation téléphone, recherche floue et sémantique, projet rangé par version et état de suivi, paramètres et prompts de page, accueil et parcours de l'atelier, skills des techniques, axes, clarifications, réception unifiée, carte de réception, git, graphe.

**Exclu** : le front en Rust / WebAssembly (REC-009.2) et le design system Rust : **v10**.

## Cas d'usage prioritaires

1. Sur son téléphone, Tibo ouvre Projet, choisit la v9 et lit l'exigence EF-174 et son test, sans ouvrir le volet.
2. Il ouvre trois documents depuis le volet : un seul onglet d'aperçu change ; il fixe celui qu'il veut garder.
3. Dans un atelier express, il coche trois techniques, corrige le skill du Free writing pour cet atelier et met un axe en pause.
4. En réception, il glisse une remarque traitée vers « validée » depuis la carte de la page, et voit la couleur de la version qui la contient.

## Besoins

| ID | Besoin | Priorité |
|---|---|---|
| [[besoins/B-024]] | Naviguer dans le wiki sans s'encombrer, sur ordinateur comme sur téléphone | must |
| [[besoins/B-025]] | Suivre un projet version par version | must |
| [[besoins/B-026]] | Régler les agents là où on s'en sert | must |
| [[besoins/B-027]] | Piloter et adapter un atelier de design thinking | should |
| [[besoins/B-028]] | Réceptionner et publier sans quitter wikik | must |

## Critères de succès de la version

Les remarques sont au statut « traitée » ; `make check` et `wikik check` passent ; `wikik stats` reste au-dessus de 95 % ; contrôle headless clair, sombre et téléphone sans erreur.

## Hypothèses et risques

- Le classement par version s'appuie sur le champ `version` déjà présent dans le frontmatter (ADR-0018).
- La recherche sémantique demande un modèle d'embeddings local : elle reste facultative et l'outil marche sans (ADR-0020).

## Décision de lancement

Accordée par Tibo le 24/10/2026 (« je valide la v9 et par contre on repousse le design système en v10 »).
