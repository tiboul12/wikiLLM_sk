---
id: CDC-v5
type: cahier-des-charges
titre: "Cahier des charges v5 (cahier de réception)"
statut: proposé
version: v5
sources: [BDR-v5-2]
---

# Cahier des charges v5

## Exigences fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/EF-120]] | Construire la carte des pages à partir du code | must | B-015, B-016 |
| [[exigences/EF-121]] | Corriger la carte sans perdre les corrections | must | B-015 |
| [[exigences/EF-122]] | Compléter la carte avec le LLM | could | B-015 |
| [[exigences/EF-123]] | Déposer une remarque sur une page | must | B-015, B-016 |
| [[exigences/EF-124]] | Traiter les remarques | must | B-015 |
| [[exigences/EF-125]] | Donner les remarques à l'agent de code | must | B-016 |
| [[exigences/EF-126]] | Ouvrir la page reçue dans l'application | should | B-015 |
| [[exigences/EF-127]] | Modifier et supprimer une remarque, avec son historique | must | B-015 |

## Exigences non fonctionnelles

Celles de la v4 s'appliquent : ENF-004 (local), ENF-007 (abonnement, pas de clé), ENF-008 (LLM isolé). Les images sont servies sans exécution possible (PNG, JPEG, GIF, WebP seulement).

## Exigences modifiées ou abandonnées par rapport à la version précédente

Aucune.
