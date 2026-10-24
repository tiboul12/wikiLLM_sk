---
id: ADR-0019
type: adr
titre: "Les fiches de technique se corrigent atelier par atelier"
statut: proposé
date: 2026-10-24
exigences: [EF-178, EF-177]
---

# ADR-0019 : les fiches de technique se corrigent atelier par atelier

## Contexte

Les consignes que le LLM reçoit à chaque technique du Design thinking (axes du free writing, récolte, 5W1H…) sont les fiches `atelier/techniques/*.md`, embarquées dans le binaire. Tibo ne les trouve nulle part et veut « les faire évoluer, les corriger pour chaque atelier » (REC-098). Le même projet mène plusieurs ateliers (EF-145) qui n'ont pas forcément le même sujet.

## Décision

- Une fiche se corrige **pour un atelier** : sa copie modifiée vit dans le dossier de l'atelier, `<dossier>/.techniques/<technique>.md` (`docs/atelier/.techniques/` pour l'atelier n° 1, `docs/atelier/parcours-2/.techniques/` pour le n° 2). Tant qu'elle n'existe pas, la fiche embarquée sert ; « revenir à l'origine » supprime la copie.
- La copie a le format complet de la fiche (frontmatter des sous-étapes, rôle, consignes, règles, exemple) ; elle est refusée si elle ne s'analyse pas ou si elle change d'identifiant. wikik la relit à chaque demande : c'est le texte réellement envoyé.
- Le dossier est caché (`.techniques`) : les copies ne sont pas indexées comme documents (deux ateliers auraient deux `free-writing`), mais elles sont versionnées avec l'atelier.
- Le mode (express ou complet) est un choix de l'**atelier**, pas de l'étape (REC-094) : en express, on coche les techniques à faire et les autres sont sautées.

## Conséquences

- L'écran de chaque étape a l'encart de sa fiche, comme les prompts système des autres pages (EF-176).
- Les fiches corrigées ne remontent pas d'elles-mêmes dans les autres ateliers : une correction utile partout se reporte dans `atelier/techniques/`.

## Alternatives écartées

- **Des skills `.claude/skills/`** : ils valent pour tout le projet, pas pour un atelier, et le format des fiches (sous-étapes en frontmatter) n'est pas celui d'un skill.
- **Un dossier visible `techniques/`** : chaque copie serait indexée avec l'identifiant de sa technique, en double d'un atelier à l'autre.
