# 🛠️ Guide Pratique : Rédaction de Notes et Création de Liens

**Dernière mise à jour** : 2026-09-25  
**Public cible** : Rédacteurs, Développeurs, Agents IA

---

## 🎯 Objectif
Structurer les notes de documentation, créer des interconnexions navigables (wikilinks) et intégrer des blocs ADR standardisés.

---

## 🔗 1. Syntaxe des liens (Wikilinks)

WikiLLM prend en charge plusieurs formats de liens qui sont automatiquement résolus dans l'éditeur, le visualiseur et le graphe :

- **Lien direct** : `[[nom-note]]` -> Crée un lien vers le fichier `nom-note.md` au même niveau ou à la racine.
- **Lien avec chemin** : `[[decisions/001-choix]]` -> Crée un lien vers le sous-dossier `decisions/001-choix.md`.
- **Lien avec alias** : `[[decisions/001-choix|ADR 001]]` -> Affiche le libellé personnalisé "ADR 001" tout en pointant vers la note cible.
- **Lien Markdown classique** : `[Titre]\(chemin/vers/note.md\)` -> Pris en charge nativement dans le rendu et dans le graphe.

---

## 🏛️ 2. Intégrer un bloc ADR dans une note

Pour documenter une décision technique directement dans une note sans créer de fichier orphelin, insérez le bloc suivant :

```markdown
> ### 🏛️ Architecture Decision (ADR)
> **Statut** : Accepté | Proposé | Déprécié  
> **Contexte** : Description de la problématique rencontrée.  
> **Décision** : Choix technique retenu et justification.  
> **Conséquences & Trade-offs** :
> - 🟢 **Avantages** : Bénéfices directs.
> - 🔴 **Inconvénients / Limites** : Contraintes acceptées.
```

---

## 📁 3. Arborescence recommandée (`wiki/`)

- `wiki/index.md` : Table des matières et point d'entrée.
- `wiki/architecture/` : Diagrammes Mermaid, spécifications globales.
- `wiki/decisions/` : ADRs majeurs ou transverses.
- `wiki/sessions/` : Comptes-rendus horodatés des échanges utilisateur / agent.
- `wiki/taches/` : Feuilles de route et backlog (`backlog.md`).

---

## 🔗 Liens utiles
- [Tutoriel prise en main](../tutorials/quickstart.md)
- [Architecture & ADR](../explanation/architecture.md)
