---
id: MESURE-02
type: mesure
titre: "Coût du contexte avec wikik sur un projet TypeScript (shelley-studio-llm)"
statut: terminé
version: v2
date: 2026-10-17
sources: [MESURE-01, BDR-v2-2, ENF-001]
---

# MESURE-02 : wikik sur shelley-studio-llm (Angular)

**Terrain** : clone de shelley-studio-llm au 17/10/2026 : 42 fichiers TypeScript, 14 054 lignes, environ 141 000 tokens de code. Le projet est presque sans doc (9 % des symboles), donc un cas défavorable.
**Unité** : tokens estimés (octets / 4). Même méthode approchée que MESURE-01.

| Mesure | Résultat |
|---|---|
| Premier index | 174 ms, 663 symboles hors champs, 419 appels certains, 54 probables |
| `carte` | 898 tokens (budget 4 000) |
| `fichier configurer.component.ts` (3 441 lignes) | 1 526 tokens, contre 38 977 pour lire le fichier : ÷ 25 |
| `impact ApiService` | 356 tokens : les 13 classes qui l'injectent |

## Ce que la mesure a fait corriger

- Les champs de classe (`signal()`, `computed()`) faussaient les compteurs : ils ne sont plus comptés comme symboles à documenter.
- Les gros composants dépassaient le budget de `fichier` (environ 4 000 tokens) : au-delà de 40 symboles, les champs et les publics sans doc sont listés par leur nom.
- `ApiService` n'avait aucun appelant : les initialiseurs de champs (`inject(ApiService)`) sont maintenant analysés.

## Limites constatées

- Les gabarits Angular (HTML) ne sont pas analysés : une méthode appelée seulement depuis le gabarit n'a pas d'appelant.
- Les appels `this.api.get(…)` ne sont résolus que si le nom de la méthode est unique dans le projet.
- Les alias de `tsconfig` (`@shelley/shared-ui`) ne sont pas résolus.
