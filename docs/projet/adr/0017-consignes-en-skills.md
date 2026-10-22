---
id: ADR-0017
type: adr
titre: "Les consignes des LLM sont des skills du projet"
statut: proposé
date: 2026-10-22
exigences: [EF-156]
---

# ADR-0017 : les consignes des LLM sont des skills du projet

## Contexte

En v7, les consignes (lecture du wiki, traitement de la réception) sont des textes courts dans `wikik.toml`, montrés tels quels dans les Paramètres. Tibo les juge « jetées comme ça » (REC-058, REC-060) et veut de vrais skills, dont celui qui apprend à documenter le code pour wikik (REC-061).

## Décision

- Chaque consigne est un skill au format Claude Code : `.claude/skills/<nom>/SKILL.md` du projet, frontmatter `name` et `description`, corps en sections. Ce sont les textes réellement envoyés : wikik lit le fichier à chaque appel.
- wikik embarque le texte d'origine de chaque skill ; tant que le fichier n'existe pas, c'est lui qui sert. Modifier un skill dans les Paramètres écrit le fichier ; « Revenir à l'origine » le supprime.
- Skills livrés : `wikik` (lire le projet et documenter le code, l'ancien skill `skill/SKILL.md`), `wikik-traiter-reception` (consigne envoyée à l'agent), `wikik-lecture` (préambule des demandes du LLM et instructions du serveur MCP), `wikik-clarification` (juger la question, proposer des pistes), `wikik-structure-<section>` (comment sont structurés les documents d'une section : besoins, exigences, tests, ADR, specs, BDR).
- Étant dans `.claude/skills/`, ces skills sont aussi ceux de l'agent de développement qui travaille dans le projet.

## Conséquences

- `consigne_lecture` et `consigne_traitement` de `wikik.toml` disparaissent ; une valeur trouvée est reprise dans le skill correspondant à la première écriture.
- Un projet versionne ses consignes avec son code.

## Alternatives écartées

- **Garder les consignes dans `wikik.toml`** : une chaîne TOML n'est ni lisible ni réutilisable par l'agent.
- **Un dossier propre à wikik** (`.wikik/consignes/`) : `.wikik/` n'est pas versionné et l'agent ne les verrait pas.
