---
id: CDC-v6
type: cahier-des-charges
titre: "Cahier des charges v6 (un seul serveur, accueil et sections)"
statut: proposé
version: v6
sources: [BDR-v6-2]
---

# Cahier des charges v6

## Exigences fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/EF-130]] | Un seul serveur, lancé par `wikik` | must | B-017 |
| [[exigences/EF-131]] | Accueil, barre d'activité et accueils de section | must | B-017 |

## Exigences non fonctionnelles

ENF-004 (local, pas d'exécution de contenu) et les gardes d'écriture de l'atelier s'appliquent au serveur unique.

## Exigences modifiées ou abandonnées par rapport à la version précédente

- [[exigences/EF-100]] (lancer le serveur de l'atelier) : l'atelier est une section du serveur unique (ADR-0012).
- SPEC-09 R1, SPEC-20 et SPEC-23 R1 : `wikik serve`, `wikik atelier`, `wikik reception` remplacés par `wikik`.
