---
id: "CLAR-002"
type: "clarification"
titre: "Clarification de B-001"
statut: "brouillon"
etat: "ouverte"
pistes: [{"justification":"B-001 désigne « l'agent de code (et Tibo) ». Or BDR-v1-2 sépare deux cas d'usage : le n° 2 (l'agent obtient une carte en début de session, en quelques centaines de lignes) et le n° 5 (Tibo lit dans le navigateur le code et les documents reliés), ce dernier étant porté par B-006. Si la frontière reste floue, B-001 et B-006 risquent d'engendrer des exigences en double. Les exigences liées à B-001 (EF-020, ENF-001) parlent de budget de tokens, donc d'un lecteur agent.","piste":"Préciser le « qui » : B-001 vise d'abord l'agent de code, qui ne doit pas ouvrir les fichiers source. Ce que Tibo lit relève de B-006 (lire le projet documenté dans un navigateur). On peut garder « et Tibo » seulement si la carte et la fiche d'un fichier doivent aussi lui servir de point d'entrée dans le wiki.","sources":["B-001","BDR-v1-2","EF-020","ENF-001"]}]
question: "besoin · Comprendre la structure d'un projet sans relire ses fichiers source"
sources: ["B-001"]
sujet: "B-001"
---

# Clarification de B-001

**Question** : besoin · Comprendre la structure d'un projet sans relire ses fichiers source

## Pistes

- **Préciser le « qui » : B-001 vise d'abord l'agent de code, qui ne doit pas ouvrir les fichiers source. Ce que Tibo lit relève de B-006 (lire le projet documenté dans un navigateur). On peut garder « et Tibo » seulement si la carte et la fiche d'un fichier doivent aussi lui servir de point d'entrée dans le wiki.** — B-001 désigne « l'agent de code (et Tibo) ». Or BDR-v1-2 sépare deux cas d'usage : le n° 2 (l'agent obtient une carte en début de session, en quelques centaines de lignes) et le n° 5 (Tibo lit dans le navigateur le code et les documents reliés), ce dernier étant porté par B-006. Si la frontière reste floue, B-001 et B-006 risquent d'engendrer des exigences en double. Les exigences liées à B-001 (EF-020, ENF-001) parlent de budget de tokens, donc d'un lecteur agent. (sources : B-001, BDR-v1-2, EF-020, ENF-001)
