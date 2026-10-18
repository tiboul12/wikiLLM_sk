---
id: BDR-v5-2
type: bdr-definition
titre: "Définition du besoin v5 (cahier de réception)"
statut: proposé
version: v5
sources: [BDR-v5-1]
---

# BDR v5, partie 2 : définition du besoin

## Objectifs mesurables

- Sans aucune configuration, la carte des pages de wikik, d'ATS et de shelley-studio-llm est trouvée par le scan (toutes les pages qu'un utilisateur atteint).
- Une remarque avec capture se saisit en moins de 30 secondes.
- Toutes les remarques d'une campagne se retrouvent, rangées par page, et chacune finit traitée, reportée ou rejetée avec une raison.

## Périmètre (inclus / exclu)

**Inclus** : carte des pages construite à partir du code, corrigeable ; remarques en cartes avec images collées ; traitement des remarques ; accès de l'agent de code par MCP.

**Exclu** : tests automatisés, plusieurs réceptionnaires simultanés, export PDF du cahier.

## Cas d'usage prioritaires

1. Tibo lance la réception dans un projet, voit la liste de ses pages, en ouvre une et y dépose ses remarques avec captures.
2. En fin de campagne, Tibo et l'agent repassent les remarques ouvertes ; l'agent ouvre directement le code de la page et applique la correction.

## Besoins

| ID | Besoin | Priorité |
|---|---|---|
| [[besoins/B-015]] | Faire ses retours d'utilisateur de façon ordonnée, page par page | must |
| [[besoins/B-016]] | Retrouver le code d'un écran à partir d'une remarque | must |

## Critères de succès de la version

Ceux des objectifs, vérifiés sur wikik, ATS (clone) et shelley-studio-llm.

## Hypothèses et risques

- Les façons de déclarer les écrans sont nombreuses : quatre détecteurs couvrent les terrains connus ; le LLM complète les autres.
- Une heuristique peut proposer une page qui n'en est pas une : Tibo la masque, et le scan suivant s'en souvient.

## Décision de lancement

Accordée par Tibo le 18/10/2026 (« oui lance la v5 »).
