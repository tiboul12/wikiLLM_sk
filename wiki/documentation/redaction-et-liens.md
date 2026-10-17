# 🛠️ Guide : Rédaction de Notes, Wikilinks & Blocs ADR

**Dernière mise à jour** : 2026-09-28  
**Public cible** : Rédacteurs, Développeurs, Agents IA

---

## 🔗 1. Syntaxe des liens (Wikilinks)

Le moteur de rendu et de graphe prend en charge l'ensemble des syntaxes de liens :

- **Lien direct** : `[[nom-note]]` -> Résout `nom-note.md` à la racine ou dans les sous-dossiers.
- **Lien avec sous-dossier** : `[[documentation/multi-projets]]` -> Cible directement la note du sous-dossier.
- **Lien avec alias** : `[[decisions/001-choix-technologiques|ADR 001]]` -> Affiche "ADR 001" comme texte de lien cliquable.
- **Lien Markdown classique** : `[Titre](chemin/vers/note.md)`.

---

## ⚓ 2. Hooks d'Ancrage pour Agents IA

Chaque titre, bloc de code, tableau ou citation dispose d'un bouton **⚓ Hook** au survol dans l'interface web :
- Cliquez sur **⚓ Hook** pour copier dans le presse-papier la référence absolue (ex: `@wiki:wikiK/architecture/vue-ensemble#flux-de-travail`).
- Collez cette ancre directement dans vos prompts pour désigner à votre assistant IA la section exacte à inspecter ou éditer.

---

## 🏛️ 3. Intégration d'un Bloc ADR (Architecture Decision Record)

Dans une note de documentation ou un fichier source, utilisez le modèle standard suivant :

```markdown
> ### 🏛️ Architecture Decision (ADR)
> **Statut** : Accepté | Proposé | Déprécié  
> **Contexte** : Problématique rencontrée ou besoin technique.  
> **Décision** : Solution technique retenue et justification.  
> **Conséquences & Trade-offs** :
> - 🟢 **Avantages** : Bénéfices directs.
> - 🔴 **Limites / Contraintes** : Compromis acceptés.
```

---

## 🔗 Liens Utiles
- [[documentation/guide|Guide Général]]
- [[architecture/code-map|Cartographie du Code]]
