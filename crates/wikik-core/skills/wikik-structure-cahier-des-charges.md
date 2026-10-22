---
name: wikik-structure-cahier-des-charges
description: Comment rédiger le cahier des charges d'une version majeure de wikik — tableau des exigences reliées aux besoins, exigences modifiées. Utilisé quand le LLM ou l'agent prépare une version majeure.
---

# Structure d'un cahier des charges

Un fichier par version : `docs/projet/majeures/<version>/cahier-des-charges.md`.

## Corps

- **Exigences fonctionnelles** : tableau `ID | Exigence | Priorité | Besoins`, une ligne par `EF-xxx`, chaque exigence dans son fichier.
- **Exigences non fonctionnelles** : celles qui s'appliquent.
- **Exigences modifiées ou abandonnées** par rapport à la version précédente.

Chaque besoin du BDR est couvert par au moins une exigence ; chaque exigence par au moins un test.
