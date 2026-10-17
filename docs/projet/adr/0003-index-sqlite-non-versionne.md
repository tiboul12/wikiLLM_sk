---
id: ADR-0003
type: adr
titre: "Index SQLite non versionné, déterministe et reconstructible"
statut: accepté
date: 2026-10-17
remplace: []
exigences: [EF-001, EF-002, EF-003, ENF-002]
---

# ADR-0003 : Index SQLite non versionné, déterministe et reconstructible

## Contexte

L'agent interroge l'index à la demande (D5). Plusieurs personnes devront un jour obtenir le même index à partir du même commit, sans conflit de merge.

## Décision

- Index dans **`.wikik/index.sqlite`** (SQLite inclus dans le binaire, FTS5 pour la recherche), **ignoré par git**.
- **Déterministe** : le dump JSON (SPEC-02) est identique pour un même état du dépôt ; aucun horodatage ni identifiant interne dans les données exposées.
- **Reconstructible** : un index d'une autre version de schéma est supprimé et reconstruit, jamais migré.
- Les seules informations qui doivent survivre (référence de fraîcheur de la doc) viennent de git (ADR-0006), pas d'un fichier versionné.

## Conséquences

- Aucun conflit de merge ; un clone neuf reconstruit l'index en quelques secondes.
- La CI future pourra mettre l'index en cache par commit.

## Alternatives écartées

- **Index versionné** (JSON ou SQLite dans le dépôt) : conflits à chaque PR.
- **Fichiers JSON à plat** : pas de requêtes ciblées ni de recherche plein texte.
