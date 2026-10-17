---
id: MESURE-03
type: mesure
titre: "Atelier v4 : répétition du parcours Express et rappel sur ATS"
statut: en cours
version: v4
date: 2026-10-18
sources: [BDR-v4-2, SPIKE-02, EF-104, EF-110]
---

# MESURE-03 : l'atelier avec le vrai LLM

**Fournisseur** : `claude -p` (abonnement Claude Code), branché sur le serveur MCP de wikik, outils intégrés désactivés.
**Statut** : répétition et rappel mesurés ; le lancement réel du projet « météo » reste à faire par Tibo (section 3).

## 1. Répétition générale du parcours Express

**Terrain** : projet jetable « MyRandonnée » (le projet d'exemple des fiches), dépôt vide, piloté par l'API de l'atelier. Pitch et textes libres écrits pour l'essai ; politique de choix fixe : garder le nombre conseillé par la fiche, sinon tout.

| Mesure | Résultat |
|---|---|
| Étapes parcourues | les 11 (pitch → roadmap), Express |
| Appels au LLM | 23, aucun échec, aucune relance |
| Durée des appels | 5,7 à 16,7 s, 9,8 s en moyenne ; 225 s pour tout le parcours (hors temps de Tibo) |
| Propositions | 92 : 77 gardées, 15 rejetées ; 0 verbatim écarté |
| Produit | 24 unités de sens, 12 idées, 3 analyses, 1 storyboard (5 vignettes, 4 artefacts), 10 besoins, 8 exigences, BDR 1 et 2, cahier des charges, roadmap |
| `wikik check` sur le dépôt produit | 0 erreur, 0 avertissement |
| Consommation | fenêtre de 5 h : de 74 % à 98 % sur la session (développement compris) ; 7 jours : 67 % → 70 % |

Qualité observée : le pitch marque « à préciser » ce que la description ne dit pas ; les verbatims sont extraits mot pour mot ; chaque exigence a un critère d'acceptation chiffré (« alerte perçue en moins de 30 s dans 95 % des essais ») ; chaque besoin cite un verbatim dans son « pourquoi ».

## 2. Rappel pendant le développement (ATS, `AC-019`)

**Terrain** : clone local d'ATS (le dépôt n'est pas modifié). Question : une suggestion `add_keyword` est-elle permise quand l'annonce exige une compétence absente de la base de connaissances, au regard de C17 ?

| Mesure | Résultat |
|---|---|
| Ouverture | `CLAR-001` créée sur `AC-019` ; la page d'`AC-019` la liste dans « Documents qui y renvoient » |
| Pistes | 3 en 11,8 s, chacune justifiée par `AC-019`, `C17`, le CDC v1 ou le backlog |
| Apport | le LLM a relevé une vraie lacune : `AC-019` ne dit rien du cas d'une compétence absente ; il propose un critère testable et un cas de test négatif |
| Par l'agent de code | `claude -p` avec le seul serveur MCP de wikik ouvre une clarification par `atelier_rappel` et rend l'URL : 5,9 s |

La conclusion reste à Tibo (elle n'a pas été jouée à sa place).

## 3. À faire avec Tibo

- **Lancement réel du projet « météo »** : pitch et free writing écrits par Tibo, parcours Express dans l'interface ; mesurer la durée de la séance, les propositions gardées et rejetées, le ressenti.
- **Rappel sur le vrai dépôt ATS** : avec son accord (création de `docs/atelier/clarifications/` dans ATS), rejouer la clarification d'`AC-019` et la conclure.

## Ce que la mesure a fait corriger

- Le champ `probleme` du pitch (du texte) était lu comme un lien : les champs de lien de l'atelier ignorent une valeur qui n'a pas la forme d'un identifiant.
- Le brouillon de pitch n'avait pas accès à la description : la demande au LLM transporte maintenant le texte saisi.
