---
id: BDR-v9-1
type: bdr-analyse
titre: "Analyse du besoin v9 (retours de la réception de la v8)"
statut: proposé
version: v9
sources: [BDR-v8-2, REC-002, REC-003, REC-062, REC-069, REC-082, REC-083, REC-084, REC-085, REC-086, REC-087, REC-088, REC-089, REC-090, REC-091, REC-092, REC-093, REC-094, REC-095, REC-096, REC-097, REC-098, REC-099, REC-100, REC-101.2, REC-102, REC-103, REC-104.3, REC-105, REC-106, REC-107, REC-108, REC-109, REC-110, REC-111, REC-112, REC-113, REC-114, REC-115]
---

# BDR v9, partie 1 : analyse du besoin

> Aucune solution technique dans ce document. Matière : les 38 remarques ouvertes du cahier de réception au 24/10/2026, saisies par Tibo en recevant la v8 (dont 6 remarques de la v8 rouvertes « à reprendre »).

## Contexte

Troisième réception complète. Quatre remarques sont bloquantes (Paramètres, onglets, skills de l'atelier, navigation téléphone). Tibo utilise maintenant wikik **sur son téléphone** (captures REC-091 à 100) et sur un second projet (meteoKry) : les manques de navigation et de structure se voient davantage.

## Personnes concernées

Tibo, qui définit le besoin, relit, réceptionne et fait développer par un agent ; plus tard, l'équipe.

## Situation actuelle et irritants (avec preuves)

| Irritant | Preuve |
|---|---|
| Les onglets s'accumulent sans logique naturelle et ont des défauts. | REC-069 (« trouve le cas d'UX design qui utilise des onglets »), REC-088 (barre de défilement sur les titres), REC-089 (mauvais contenu après fermeture). |
| Sur téléphone, on ne va pas plus loin que l'accueil d'une section. | REC-099 (bloquant), REC-100 (titres absents). |
| La documentation d'une version est éparpillée par catégorie, illisible pour Tibo comme pour le modèle. | REC-101.2 (« identifier en une fois toute la structure d'une version »), REC-103, REC-102 (cascades sans repli). |
| Le Projet résume au lieu de suivre. | REC-090 (« un état de suivi et pas un récap »), REC-083 (rien sur l'accueil). |
| Paramètres mélange la connexion et les consignes, et a des champs inutiles. | REC-062 (bloquant), REC-104.3 (prompts à mettre sur la page qui les utilise), REC-105 (listes mal alignées), REC-106 (« à quoi ça sert ? »), REC-107 (« demande non sollicitée »). |
| Les consignes de l'atelier ne sont ni visibles ni adaptables. | REC-098 (bloquant). |
| L'atelier s'ouvre au milieu d'un parcours et se pilote mal. | REC-091 (pas d'accueil), REC-094 (express ou complet par technique), REC-093 (pas modifiable sur téléphone), REC-095 (pas de pause), REC-096 (propositions d'axes déplacées). |
| La clarification perd le contexte et s'éparpille. | REC-114, REC-115, REC-112, REC-113. |
| La réception a deux écrans différents et ses états sont confus. | REC-085, REC-086, REC-082 (couleur par version non visible), REC-092 (capture du presse-papier sur téléphone), REC-084 (page « X » sans objet). |
| La section Git n'explique pas ce qu'elle montre. | REC-108 (fichiers non commités sans lien), REC-109, REC-110 (branches à plat), REC-111 (procédure de publication absente). |
| La recherche ne comprend pas ce qu'on cherche. | REC-087 (« smart recherche »), REC-097 (recherche sémantique, à la grepai). |
| Le graphe se lit mal dans les zones denses. | REC-002 (zoom lent), REC-003 (liens non mis en évidence). |

## Contraintes

- Une seule commande `wikik` (ADR-0012) ; pas d'« usine à gaz ».
- Les données restent des fichiers du dépôt (ADR-0010), tout reste local (ENF-004).
- Écriture et commandes réservées au propriétaire (ADR-0013).

## Hors sujet

- La réécriture du front en Rust / WebAssembly (REC-009.2) et le design system Rust : **v10**.
