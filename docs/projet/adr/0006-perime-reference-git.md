---
id: ADR-0006
type: adr
titre: "Doc périmée : référence prise dans git, comparaison de hash normalisés"
statut: accepté
date: 2026-10-17
remplace: []
exigences: [EF-010, EF-015, ENF-002]
---

# ADR-0006 : Doc périmée, référence prise dans git

## Contexte

Il faut savoir si le code d'une fonction a changé « depuis » sa doc, sans fichier d'état versionné (conflits de merge) et sans fausse alerte quand le code est seulement reformaté.

## Décision

La référence d'un symbole est son état **au dernier commit qui a touché son bloc de doc** (`git blame` sur les lignes du bloc). On compare les hash normalisés de signature et de corps entre ce commit et l'état actuel. Détail : SPEC-04. Hors dépôt git, repli sur un état local dans `.wikik/`.

## Conséquences

- Déterministe pour un état de dépôt donné, donc réutilisable tel quel en CI.
- Toucher le bloc (y compris `@relu`) le rend frais : c'est le geste de relecture.
- Coût d'un `git blame` par fichier documenté, mis en cache par commit.

## Alternatives écartées

- **Fichier d'état versionné** : conflits de merge.
- **Dates de commit du corps et du bloc** : fausses alertes au reformatage.
