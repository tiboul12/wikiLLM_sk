---
id: CDC-v9
type: cahier-des-charges
titre: "Cahier des charges v9 (retours de la réception de la v8)"
statut: proposé
version: v9
sources: [BDR-v9-2]
---

# Cahier des charges v9

## Exigences fonctionnelles

| ID | Exigence | Priorité | Besoins |
|---|---|---|---|
| [[exigences/EF-169]] | Onglets en aperçu ou fixés | must | B-024 |
| [[exigences/EF-170]] | Cascades repliables, page Cascade retirée | should | B-024 |
| [[exigences/EF-171]] | Navigation complète sur téléphone | must | B-024 |
| [[exigences/EF-172]] | Recherche floue au fil de la frappe | should | B-024 |
| [[exigences/EF-173]] | Recherche sémantique locale | could | B-024 |
| [[exigences/EF-174]] | Projet rangé par version | must | B-025 |
| [[exigences/EF-175]] | État de suivi du projet | must | B-025 |
| [[exigences/EF-176]] | Paramètres réduits à la connexion et aux prompts généraux | must | B-026 |
| [[exigences/EF-177]] | Accueil et parcours de l'atelier | should | B-027 |
| [[exigences/EF-178]] | Skills des techniques modifiables par atelier | must | B-027 |
| [[exigences/EF-179]] | Axes : pause et propositions en tête | should | B-027 |
| [[exigences/EF-180]] | Clarifier depuis la fiche, avec son contexte | should | B-027 |
| [[exigences/EF-181]] | Une seule interface de réception | must | B-028 |
| [[exigences/EF-182]] | Carte de réception sans fausse page | should | B-028 |
| [[exigences/EF-183]] | Git lisible et procédure de publication | should | B-028 |
| [[exigences/EF-184]] | Graphe : zoom progressif et mise en évidence | should | B-001 |

## Exigences non fonctionnelles

ENF-004 (local) s'applique, y compris à la recherche sémantique ; l'écriture de fichiers depuis le navigateur suit ADR-0013 et ADR-0016.

## Exigences modifiées ou abandonnées par rapport à la version précédente

- EF-151 : les onglets gagnent l'aperçu (EF-169).
- EF-155, EF-156 : Paramètres ne garde que la connexion et les prompts généraux ; les prompts de page vont sur leur page (EF-176).
- EF-157, EF-160 : le volet Projet se range par version (EF-174) ; la page Cascade disparaît (EF-170).
- EF-165 : la validation passe sur un curseur d'états, dans une interface unique (EF-181).
