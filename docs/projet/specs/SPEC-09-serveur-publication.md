---
id: SPEC-09
type: spec
titre: "Serveur de prévisualisation et publication"
statut: proposé
version: v1
exigences: [EF-040, ENF-004, ENF-005]
decisions: [ADR-0004, ADR-0005]
---

# SPEC-09 : Serveur de prévisualisation et publication

## Problème

Servir le wiki en local à partir de l'index, sans exposer d'écriture ni dépendre d'internet ; préparer la publication statique de la phase équipe sans la construire.

## Exigences

1. **R1** `wikik serve` écoute sur `127.0.0.1:8484` par défaut. Une autre adresse que la boucle locale exige `--autoriser-reseau` (EF-040). Pour le réseau local, `--reseau`, ou `[serveur] reseau = true` dans `wikik.toml` une fois pour toutes, fait écouter sur toutes les interfaces ; le serveur affiche alors l'adresse à ouvrir depuis un autre poste. Même règle pour `wikik atelier` (port 8485).
2. **R2** API **en lecture seule** (`GET` uniquement) ; toute autre méthode répond 405.
3. **R3** Chaque requête d'API met l'index à jour (incrémental), comme les requêtes CLI : le wiki reflète le disque sans redémarrage.
4. **R4** Le front est **embarqué** dans le binaire (`rust-embed`) ; les routes hors `/api/` renvoient `index.html` (application monopage).
5. **R5** En-têtes de sécurité : `Content-Security-Policy` sans source externe, `X-Content-Type-Options: nosniff`.
6. **R6** Les chemins de fichiers demandés sont ceux de l'index : aucun accès disque à partir d'un chemin fourni par le client.

## Proposition

| Route | Réponse |
|---|---|
| `GET /api/projet` | Nom, statistiques, arbre des fichiers, documents (id, titre, type, statut) |
| `GET /api/fichier?chemin=` | Page fichier : en-tête, cellules (symbole, doc, statut, code, voisins, balises) |
| `GET /api/document?id=` | Page document : frontmatter, corps, table des liens résolus, symboles qui le citent |
| `GET /api/graphe?centre=&profondeur=` | Nœuds et arêtes autour d'un symbole ou d'un document ; sans centre : graphe des documents |
| `GET /api/cherche?q=` | Résultats FTS5 |
| `GET /api/check` | Rapport de `check` (JSON de SPEC-04) |

Serveur : `axum` + `tokio`, index partagé derrière un `Mutex`, appels SQLite dans `spawn_blocking`.

**Publication (phase équipe, backlog)** : le même front, servi en fichiers statiques, lira un export JSON de l'index ou le moteur de requêtes compilé en WASM. Rien à construire en v1 ; l'API ci-dessus est pensée pour être remplaçable par ces fichiers.

## Critères d'acceptation

1. `wikik serve --adresse 0.0.0.0` échoue sans `--autoriser-reseau`.
2. `POST /api/fichier` → 405.
3. Modifier un bloc `///` puis recharger la page du fichier montre la nouvelle doc.
