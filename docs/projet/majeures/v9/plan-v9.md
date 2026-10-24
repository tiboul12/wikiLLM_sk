---
id: PLAN-v9
type: plan
titre: "Plan v9 : traitement des retours de la réception de la v8"
statut: proposé
version: v9
sources: [BDR-v9-2, CDC-v9]
---

# Plan v9 : traitement des retours de la réception de la v8

**En une phrase** : les 38 remarques ouvertes du cahier de réception au 24/10/2026 sont traitées par lots, sur la branche `v9-retours` (partie de `v8-retours`, non fusionnée), puis fusionnées dans `master` après la réception de Tibo.

## Numérotation

La v9 était réservée au front en Rust / WebAssembly (REC-009.2). Cette réception passe devant : **la v9 traite les retours, le front Rust et le design system deviennent la v10** (décision de Tibo, 24/10/2026).

## Lots

| Phase | Lot | Remarques | Exigences |
|---|---|---|---|
| 1 | Corrections rapides | REC-084, 103, 112, 109, 088, 089, 100 | EF-182, EF-170, EF-180, EF-183, EF-169, EF-171 |
| 2 | Onglets et cascades | REC-069, 102 | EF-169, EF-170 |
| 3 | Paramètres et prompts de page | REC-062, 104.3, 105, 106, 107 | EF-176 |
| 4 | Projet par version | REC-101.2, 090, 083, 099, 114 | EF-174, EF-175, EF-171, EF-180 |
| 5 | Réception | REC-082, 085, 086, 092 | EF-181 |
| 6 | Graphe | REC-002, 003 | EF-184 |
| 7 | Design thinking | REC-091, 098, 094, 093, 095, 096, 113, 115 | EF-177, EF-178, EF-179, EF-180, EF-172 |
| 8 | Git | REC-108, 110, 111 | EF-183 |
| 9 | Recherche | REC-087, 097 | EF-172, EF-173 |

Ordre : les corrections rapides d'abord (risque nul), puis la coquille (onglets), puis les bloquants (paramètres, projet, atelier), la recherche sémantique en dernier (la plus incertaine).

## Choix par remarque (quand la remarque laisse un choix)

- **REC-069** : onglet d'aperçu à la VS Code : un clic remplace l'aperçu (titre en italique), double-clic sur l'onglet, modification, Ctrl+clic ou clic du milieu le fixe.
- **REC-084** : la page « X » venait de l'exemple `'/x'` d'un bloc de doc du scan des routes ; le scan ignore les commentaires.
- **REC-101.2** : arbre par version (ADR-0018) : demande d'évolution, BDR analyse, BDR définition, cahier des charges → besoins → exigences → tests ; ADR, specs, plan de la version à côté ; ce qui n'a pas de version reste dans « Transverse ».
- **REC-110** : l'historique paraissait linéaire parce que `v7-retours` et `v8-retours` ne sont pas fusionnées ; le dessin des branches le montre.
- **REC-097** : ADR-0019 avant le code.

## Suivi

| Phase | État | Date |
|---|---|---|
| 0 BDR, CDC, exigences, plan | ✅ | 2026-10-24 |
| 1 Corrections rapides | ✅ scan des routes sans les commentaires (plus de page « X ») ; page Cascade retirée ; « Pistes » ; branches sans puce, locales et distantes ouvertes pareil ; onglets plus hauts sans barre de défilement visible (molette) ; fermer l'onglet actif affiche son voisin ; icônes seules dans la barre du téléphone | 2026-10-24 |
