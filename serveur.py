#!/usr/bin/env python3
"""
Serveur Web Universel Multi-Projets pour Wiki de Projet (LLM-Wiki).

Ce module fournit un serveur HTTP autonome (Single-Page Application et API REST)
permettant de visualiser, éditer, explorer et lier des notes Markdown au format Wiki.

Architecture Decision Record (ADR) intégré :
- Voir les blocs ADR intégrés dans les fonctions clés (zéro-dépendance, routage SPA, graphe de relations).
"""

from __future__ import annotations

import json
import os
import re
import socket
import subprocess
import sys
import urllib.parse
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union

PORT_DEFAUT: int = 8000
SCRIPT_DIR: str = os.path.dirname(os.path.abspath(__file__))
CONFIG_FILE: str = os.path.join(SCRIPT_DIR, ".wikillm_projets.json")

# ==============================================================================
# GESTION DU REGISTRE DES PROJETS
# ==============================================================================

def charger_projets_enregistres() -> Dict[str, Dict[str, str]]:
    """Charge le registre des projets sauvegardés depuis le fichier de configuration JSON.

    Returns:
        Dict[str, Dict[str, str]]: Dictionnaire associant l'identifiant du projet à ses métadonnées
        (ex: {'nom': 'wikiK', 'chemin': '/path/to/wikiK'}).
    """
    if os.path.exists(CONFIG_FILE):
        try:
            with open(CONFIG_FILE, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            pass
    # Projet par défaut = dossier courant ou dossier du script
    cwd = os.getcwd()
    nom_defaut = os.path.basename(cwd.rstrip(r"\/")) or "projet"
    return {nom_defaut: {"nom": nom_defaut, "chemin": cwd}}


def sauvegarder_projets_enregistres(projets: Dict[str, Dict[str, str]]) -> None:
    """Sauvegarde le registre des projets dans le fichier de configuration JSON.

    Args:
        projets (Dict[str, Dict[str, str]]): Dictionnaire des projets à persister.
    """
    try:
        with open(CONFIG_FILE, "w", encoding="utf-8") as f:
            json.dump(projets, f, indent=2, ensure_ascii=False)
    except Exception as e:
        print(f"Erreur sauvegarde config projets: {e}")


REGISTRE_PROJETS: Dict[str, Dict[str, str]] = charger_projets_enregistres()


# ==============================================================================
# UTILITAIRES ET MANIPULATION DU SYSTÈME DE FICHIERS
# ==============================================================================

def normaliser_chemin(chemin_brut: Optional[str]) -> str:
    """Normalise un chemin de fichier pour Linux, macOS et Windows.

    Effectue l'expansion du tilde `~` et résout le chemin absolu.

    Args:
        chemin_brut (Optional[str]): Chemin d'accès brut passé en entrée.

    Returns:
        str: Chemin absolu normalisé ou chaîne vide si l'entrée est None ou vide.
    """
    if not chemin_brut:
        return ""
    chemin = chemin_brut.strip()
    if chemin.startswith("~"):
        chemin = os.path.expanduser(chemin)
    return os.path.abspath(chemin)


def scanner_dossier_racine(chemin_racine: str) -> Dict[str, Any]:
    """Scanne un répertoire racine pour détecter les projets et sous-projets contenant un dossier wiki.

    <!-- Embedded Architecture Decision Record (ADR Block) -->
    # 🏛️ Architecture Decision (ADR)
    # Statut : Accepté
    # Contexte : Nécessité de détecter automatiquement les projets existants sans configuration manuelle.
    # Décision : Parcours récursif limité (profondeur 1-2) avec détection du sous-dossier 'wiki/'.
    # Conséquences & Trade-offs :
    # - 🟢 Avantages : Découverte instantanée, pas de dépendance de base de données.
    # - 🔴 Limites : Ne détecte pas les arborescences arbitrairement profondes au-delà de 2 niveaux.

    Args:
        chemin_racine (str): Chemin du répertoire à analyser.

    Returns:
        Dict[str, Any]: Dictionnaire contenant l'erreur éventuelle, le chemin racine résolu,
        et la liste des projets détectés avec le nombre de notes `.md`.
    """
    racine = normaliser_chemin(chemin_racine)
    resultats: List[Dict[str, Any]] = []

    if not os.path.isdir(racine):
        return {"erreur": f"Le dossier n'existe pas : {chemin_racine}", "projets": []}

    try:
        # Vérifie si la racine elle-même contient un dossier wiki
        if os.path.isdir(os.path.join(racine, "wiki")):
            nom_p = os.path.basename(racine.rstrip(r"\/"))
            resultats.append({
                "id": nom_p,
                "nom": nom_p,
                "chemin": racine,
                "a_wiki": True,
                "nb_notes": compter_notes(os.path.join(racine, "wiki"))
            })

        # Scanne les sous-dossiers directs (profondeur 1 à 2)
        with os.scandir(racine) as entries:
            for entry in sorted(entries, key=lambda e: e.name.lower()):
                if entry.is_dir() and not entry.name.startswith("."):
                    dossier_wiki = os.path.join(entry.path, "wiki")
                    a_wiki = os.path.isdir(dossier_wiki)
                    nb = compter_notes(dossier_wiki) if a_wiki else 0
                    resultats.append({
                        "id": entry.name,
                        "nom": entry.name,
                        "chemin": entry.path,
                        "a_wiki": a_wiki,
                        "nb_notes": nb
                    })
    except Exception as e:
        return {"erreur": str(e), "projets": []}

    return {"erreur": None, "racine": racine, "projets": resultats}


def compter_notes(dossier_wiki: str) -> int:
    """Compte récursivement le nombre total de notes Markdown (`.md`) dans un répertoire wiki.

    Args:
        dossier_wiki (str): Chemin vers le dossier wiki.

    Returns:
        int: Nombre total de fichiers `.md` trouvés.
    """
    total = 0
    if not os.path.isdir(dossier_wiki):
        return 0
    for _, _, fichiers in os.walk(dossier_wiki):
        total += sum(1 for f in fichiers if f.endswith(".md"))
    return total


def initialiser_wiki_projet(chemin_projet: str) -> str:
    """Initialise l'arborescence standard d'un wiki au sein d'un répertoire de projet.

    Crée les sous-dossiers fondamentaux : `architecture/`, `decisions/`, `sessions/`, `taches/`
    ainsi que la page d'accueil `index.md` si absente.

    Args:
        chemin_projet (str): Chemin du projet dans lequel initialiser le wiki.

    Returns:
        str: Chemin absolu vers le dossier wiki créé ou existant.
    """
    chemin = normaliser_chemin(chemin_projet)
    nom_projet = os.path.basename(chemin.rstrip(r"\/"))
    dossier_wiki = os.path.join(chemin, "wiki")

    os.makedirs(dossier_wiki, exist_ok=True)
    for sous in ["architecture", "documentation", "decisions", "sessions", "taches"]:
        os.makedirs(os.path.join(dossier_wiki, sous), exist_ok=True)

    index_path = os.path.join(dossier_wiki, "index.md")
    if not os.path.exists(index_path):
        with open(index_path, "w", encoding="utf-8") as f:
            f.write(f"""# 📖 Wiki de {nom_projet}

Bienvenue dans la documentation vivante du projet **{nom_projet}**.

## 🗂️ Sommaire Général
- **[[architecture/vue-ensemble|Architecture & Spécifications]]**
- **[[architecture/code-map|Cartographie du Code (Vue 1 Passe)]]**
- **[[documentation/guide|Guide d'Utilisation]]**
- **[[decisions/001-initialisation|Registre des Décisions (ADR)]]**
- **[[sessions/session-initiale|Journal des Sessions]]**
- **[[taches/backlog|Feuille de Route & Tâches]]**
""")
    return dossier_wiki


# ==============================================================================
# INTÉGRATION & COMMANDES GIT
# ==============================================================================

def executer_git(chemin_repo: str, args: List[str]) -> Tuple[bool, str]:
    """Exécute une commande git sécurisée au sein du répertoire du projet.

    Args:
        chemin_repo (str): Chemin du répertoire du projet Git.
        args (List[str]): Arguments de la commande git.

    Returns:
        Tuple[bool, str]: Succès (True/False) et sortie standard ou message d'erreur.
    """
    try:
        res = subprocess.run(
            ["git"] + args,
            cwd=chemin_repo,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace"
        )
        return (res.returncode == 0, res.stdout if res.returncode == 0 else res.stderr)
    except Exception as e:
        return (False, str(e))


def obtenir_git_statut(chemin_repo: str) -> Dict[str, Any]:
    """Extrait le statut Git du projet (branche, modifications, fichiers non suivis).

    Args:
        chemin_repo (str): Répertoire racine du projet.

    Returns:
        Dict[str, Any]: Statut détaillé (branche, fichiers modifiés, fichiers non suivis, dernier commit).
    """
    succes, branche = executer_git(chemin_repo, ["rev-parse", "--abbrev-ref", "HEAD"])
    if not succes:
        return {"est_git": False, "erreur": "Ce projet n'est pas un dépôt Git ou Git n'est pas installé."}

    _, status_raw = executer_git(chemin_repo, ["status", "--short"])
    _, log_dernier = executer_git(chemin_repo, ["log", "-1", "--pretty=format:%h - %s (%cr)"])

    fichiers_modifies = []
    fichiers_non_suivis = []
    for line in status_raw.splitlines():
        if line.startswith("??"):
            fichiers_non_suivis.append(line[3:].strip())
        elif line.strip():
            fichiers_modifies.append(line.strip())

    return {
        "est_git": True,
        "branche": branche.strip(),
        "dernier_commit": log_dernier.strip() if log_dernier else "Aucun commit",
        "modifies": fichiers_modifies,
        "non_suivis": fichiers_non_suivis,
        "propre": len(fichiers_modifies) == 0 and len(fichiers_non_suivis) == 0
    }


def obtenir_git_historique(chemin_repo: str, fichier_relatif: Optional[str] = None, limite: int = 30) -> List[Dict[str, Any]]:
    """Extrait l'historique des commits pour le projet entier ou pour un fichier spécifique.

    Args:
        chemin_repo (str): Répertoire racine du projet.
        fichier_relatif (Optional[str]): Chemin relatif de la note au sein du wiki.
        limite (int): Nombre maximum de commits à retourner.

    Returns:
        List[Dict[str, Any]]: Liste des commits avec hash, auteur, date et message.
    """
    args = ["log", f"-n{limite}", "--pretty=format:%H|%h|%an|%ad|%ar|%s", "--date=short"]
    if fichier_relatif:
        chemin_complet_rel = os.path.join("wiki", fichier_relatif) if not fichier_relatif.startswith("wiki/") else fichier_relatif
        args.extend(["--", chemin_complet_rel])

    succes, out = executer_git(chemin_repo, args)
    if not succes or not out.strip():
        return []

    commits = []
    for line in out.strip().split("\n"):
        parts = line.split("|")
        if len(parts) >= 6:
            commits.append({
                "hash_complet": parts[0],
                "hash_court": parts[1],
                "auteur": parts[2],
                "date": parts[3],
                "date_relative": parts[4],
                "message": parts[5]
            })
    return commits


def obtenir_git_diff(chemin_repo: str, fichier_relatif: Optional[str] = None, commit_hash: Optional[str] = None) -> Dict[str, Any]:
    """Récupère le diff Git (modifications en cours ou commit spécifique).

    Args:
        chemin_repo (str): Répertoire racine du projet.
        fichier_relatif (Optional[str]): Chemin relatif de la note ciblée.
        commit_hash (Optional[str]): Hash du commit spécifique à inspecter.

    Returns:
        Dict[str, Any]: Contenu du diff au format patch.
    """
    if commit_hash:
        args = ["show", "--stat", "--patch", commit_hash]
        if fichier_relatif:
            chemin_complet_rel = os.path.join("wiki", fichier_relatif) if not fichier_relatif.startswith("wiki/") else fichier_relatif
            args.extend(["--", chemin_complet_rel])
    else:
        args = ["diff", "HEAD"]
        if fichier_relatif:
            chemin_complet_rel = os.path.join("wiki", fichier_relatif) if not fichier_relatif.startswith("wiki/") else fichier_relatif
            args.extend(["--", chemin_complet_rel])

    succes, out = executer_git(chemin_repo, args)
    return {
        "succes": succes,
        "diff": out if succes else f"Erreur diff : {out}",
        "commit": commit_hash,
        "fichier": fichier_relatif
    }


# ==============================================================================
# GESTIONNAIRE DE REQUÊTES HTTP (API + ROUTAGE SPA)
# ==============================================================================

class WikiHandler(SimpleHTTPRequestHandler):
    """Gestionnaire HTTP personnalisé pour l'API REST et le service de l'application Web SPA."""

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        """Initialise le handler en ciblant le répertoire racine du script pour les assets statiques."""
        super().__init__(*args, directory=SCRIPT_DIR, **kwargs)

    def obtenir_projet_actif(self, nom_projet_ou_chemin: Optional[str] = None) -> Dict[str, str]:
        """Résout le projet actif selon le nom fourni, un chemin direct ou le projet par défaut.

        Args:
            nom_projet_ou_chemin (Optional[str]): Identifiant de projet ou chemin sur le disque.

        Returns:
            Dict[str, str]: Informations du projet (`nom` et `chemin`).
        """
        global REGISTRE_PROJETS
        if nom_projet_ou_chemin and nom_projet_ou_chemin in REGISTRE_PROJETS:
            return REGISTRE_PROJETS[nom_projet_ou_chemin]

        # Si un chemin direct a été passé
        if nom_projet_ou_chemin and (os.path.isdir(nom_projet_ou_chemin) or os.path.exists(nom_projet_ou_chemin)):
            chemin_abs = normaliser_chemin(nom_projet_ou_chemin)
            nom = os.path.basename(chemin_abs.rstrip(r"\/"))
            info = {"nom": nom, "chemin": chemin_abs}
            REGISTRE_PROJETS[nom] = info
            sauvegarder_projets_enregistres(REGISTRE_PROJETS)
            return info

        # Sinon, renvoie le premier projet enregistré
        if REGISTRE_PROJETS:
            premier_cle = next(iter(REGISTRE_PROJETS))
            return REGISTRE_PROJETS[premier_cle]

        # Par défaut : dossier courant
        cwd = os.getcwd()
        nom = os.path.basename(cwd.rstrip(r"\/"))
        return {"nom": nom, "chemin": cwd}

    def do_GET(self) -> None:
        """Traite les requêtes HTTP GET pour l'API REST et le routage de l'interface SPA.

        <!-- Embedded Architecture Decision Record (ADR Block) -->
        # 🏛️ Architecture Decision (ADR)
        # Statut : Accepté
        # Contexte : Fournir une expérience de Single Page Application (SPA) avec des URLs propres.
        # Décision : Rediriger toutes les routes sans extension de fichier vers `index.html`.
        # Conséquences & Trade-offs :
        # - 🟢 Avantages : Navigation fluide sans rechargement, URLs de type /projet/note supportées.
        # - 🔴 Limites : Nécessite que le frontend gère le routage par hash ou History API.
        """
        parsed_url = urllib.parse.urlparse(self.path)
        path = parsed_url.path.strip("/")
        params = urllib.parse.parse_qs(parsed_url.query)

        # API : Scanner un dossier
        if path == "api/scanner":
            chemin_scan = params.get("dossier", [""])[0]
            self.send_json(scanner_dossier_racine(chemin_scan))
            return

        # API : Liste des projets connus / enregistrés
        elif path == "api/projets":
            self.send_json({"projets": list(REGISTRE_PROJETS.values())})
            return

        # API : Infos du projet actif
        elif path == "api/projet_info":
            nom_p = params.get("projet", [None])[0]
            proj = self.obtenir_projet_actif(nom_p)
            dossier_wiki = os.path.join(proj["chemin"], "wiki")
            initialiser_wiki_projet(proj["chemin"])
            self.send_json({
                "nom_projet": proj["nom"],
                "repertoire": proj["chemin"],
                "dossier_wiki": dossier_wiki,
                "nb_notes": compter_notes(dossier_wiki)
            })
            return

        # API : Liste des fichiers du projet actif
        elif path == "api/fichiers":
            nom_p = params.get("projet", [None])[0]
            proj = self.obtenir_projet_actif(nom_p)
            dossier_wiki = os.path.join(proj["chemin"], "wiki")
            self.send_json(self.lister_fichiers(dossier_wiki))
            return

        # API : Contenu d'un fichier Markdown
        elif path == "api/fichier":
            nom_p = params.get("projet", [None])[0]
            rel_path = params.get("chemin", [""])[0]
            proj = self.obtenir_projet_actif(nom_p)
            dossier_wiki = os.path.join(proj["chemin"], "wiki")

            if not rel_path:
                self.send_error(400, "Chemin manquant")
                return

            full_path = os.path.abspath(os.path.join(dossier_wiki, rel_path))
            if not full_path.startswith(os.path.abspath(dossier_wiki)) or not os.path.exists(full_path):
                self.send_error(404, "Fichier non trouvé")
                return

            try:
                with open(full_path, "r", encoding="utf-8") as f:
                    contenu = f.read()
                self.send_json({"chemin": rel_path, "contenu": contenu})
            except Exception as e:
                self.send_error(500, f"Erreur lecture: {str(e)}")
            return

        # API : Graphe relationnel
        elif path == "api/graphe":
            nom_p = params.get("projet", [None])[0]
            proj = self.obtenir_projet_actif(nom_p)
            dossier_wiki = os.path.join(proj["chemin"], "wiki")
            self.send_json(self.generer_graphe(dossier_wiki))
            return

        # Routage pour l'interface Web (Single Page App)
        if "." not in path or path.endswith(".html"):
            self.path = "/index.html"

        return super().do_GET()

    def do_POST(self) -> None:
        """Traite les requêtes HTTP POST (sauvegarde, suppression, activation de projet).

        <!-- Embedded Architecture Decision Record (ADR Block) -->
        # 🏛️ Architecture Decision (ADR)
        # Statut : Accepté
        # Contexte : Autoriser l'écriture et la suppression de fichiers tout en empêchant les vulnérabilités de Directory Traversal (ex: '../../etc/passwd').
        # Décision : Résolution absolue via `os.path.abspath` et validation stricte du préfixe avec `os.path.abspath(dossier_wiki)`.
        # Conséquences & Trade-offs :
        # - 🟢 Avantages : Confinement strict et sécurisé des écritures dans le dossier wiki du projet actif.
        # - 🔴 Limites : Empêche la modification de fichiers hors du dossier wiki par conception.
        """
        parsed_url = urllib.parse.urlparse(self.path)
        path = parsed_url.path.strip("/")

        # API : Enregistrer / Activer un projet scanné
        if path == "api/activer_projet":
            data = self.lire_corps_json()
            if not data or "chemin" not in data:
                self.send_error(400, "Chemin de projet manquant")
                return

            chemin_abs = normaliser_chemin(data["chemin"])
            nom_p = data.get("nom") or os.path.basename(chemin_abs.rstrip(r"\/"))
            initialiser_wiki_projet(chemin_abs)

            REGISTRE_PROJETS[nom_p] = {"nom": nom_p, "chemin": chemin_abs}
            sauvegarder_projets_enregistres(REGISTRE_PROJETS)

            self.send_json({"statut": "succes", "projet": REGISTRE_PROJETS[nom_p]})
            return

        # API : Sauvegarder un fichier Markdown
        elif path == "api/sauvegarder":
            data = self.lire_corps_json()
            if not data or "chemin" not in data or "contenu" not in data:
                self.send_error(400, "Données invalides")
                return

            nom_p = data.get("projet")
            proj = self.obtenir_projet_actif(nom_p)
            dossier_wiki = os.path.join(proj["chemin"], "wiki")

            rel_path = data["chemin"].strip("/")
            if not rel_path.endswith(".md"):
                rel_path += ".md"

            full_path = os.path.abspath(os.path.join(dossier_wiki, rel_path))
            if not full_path.startswith(os.path.abspath(dossier_wiki)):
                self.send_error(403, "Accès refusé")
                return

            try:
                os.makedirs(os.path.dirname(full_path), exist_ok=True)
                with open(full_path, "w", encoding="utf-8") as f:
                    f.write(data["contenu"])
                self.send_json({"statut": "succes", "chemin": rel_path})
            except Exception as e:
                self.send_error(500, f"Erreur écriture: {str(e)}")
            return

        # API : Supprimer un fichier Markdown
        elif path == "api/supprimer":
            data = self.lire_corps_json()
            if not data or "chemin" not in data:
                self.send_error(400, "Chemin manquant")
                return

            nom_p = data.get("projet")
            proj = self.obtenir_projet_actif(nom_p)
            dossier_wiki = os.path.join(proj["chemin"], "wiki")

            rel_path = data["chemin"].strip("/")
            full_path = os.path.abspath(os.path.join(dossier_wiki, rel_path))
            if not full_path.startswith(os.path.abspath(dossier_wiki)) or not os.path.exists(full_path):
                self.send_error(404, "Fichier non trouvé")
                return

            try:
                os.remove(full_path)
                self.send_json({"statut": "succes"})
            except Exception as e:
                self.send_error(500, f"Erreur suppression: {str(e)}")
            return

        self.send_error(404, "Route non trouvée")

    def lire_corps_json(self) -> Optional[Dict[str, Any]]:
        """Lit et désérialise le corps d'une requête HTTP en JSON.

        Returns:
            Optional[Dict[str, Any]]: Objet JSON désérialisé ou None en cas d'erreur.
        """
        content_length = int(self.headers.get("Content-Length", 0))
        if content_length == 0:
            return None
        body = self.rfile.read(content_length).decode("utf-8")
        try:
            return json.loads(body)
        except json.JSONDecodeError:
            return None

    def send_json(self, data: Any) -> None:
        """Envoie une réponse HTTP 200 avec en-têtes JSON encodés en UTF-8.

        Args:
            data (Any): Données Python à sérialiser en JSON.
        """
        reponse = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(reponse)))
        self.end_headers()
        self.wfile.write(reponse)

    def lister_fichiers(self, dossier_wiki: str) -> List[Dict[str, Any]]:
        """Liste tous les fichiers Markdown d'un dossier wiki avec leurs métadonnées.

        Args:
            dossier_wiki (str): Chemin vers le dossier wiki.

        Returns:
            List[Dict[str, Any]]: Liste des fichiers contenant nom, chemin relatif, dossier parent et taille en octets.
        """
        arborescence: List[Dict[str, Any]] = []
        if not os.path.isdir(dossier_wiki):
            return arborescence

        for racine, _, fichiers in os.walk(dossier_wiki):
            for f in sorted(fichiers):
                if f.endswith(".md"):
                    chemin_complet = os.path.join(racine, f)
                    chemin_relatif = os.path.relpath(chemin_complet, dossier_wiki)
                    dossier = os.path.dirname(chemin_relatif)
                    arborescence.append({
                        "nom": f,
                        "chemin": chemin_relatif,
                        "dossier": dossier if dossier else "Racine",
                        "taille": os.path.getsize(chemin_complet)
                    })
        return arborescence

    def generer_graphe(self, dossier_wiki: str) -> Dict[str, List[Dict[str, str]]]:
        """Extrait les relations bidirectionnelles entre notes via les liens [[wikilink]] et [label](path.md).

        <!-- Embedded Architecture Decision Record (ADR Block) -->
        # 🏛️ Architecture Decision (ADR)
        # Statut : Accepté
        # Contexte : Visualiser les interconnexions de la pensée sous forme de graphe relationnel.
        # Décision : Extraction regex des syntaxes `[[note]]`, `[[note|titre]]` et `[titre](note.md)`.
        # Conséquences & Trade-offs :
        # - 🟢 Avantages : Analyse statique ultra-rapide sans AST Markdown lourd.
        # - 🔴 Limites : Ne vérifie pas la validité sémantique fine des blocs de code échappés.

        Args:
            dossier_wiki (str): Chemin vers le dossier wiki.

        Returns:
            Dict[str, List[Dict[str, str]]]: Structure {nodes: [...], edges: [...]} compatible vis.js / D3.
        """
        noeuds: List[Dict[str, str]] = []
        liens: List[Dict[str, str]] = []
        fichiers = self.lister_fichiers(dossier_wiki)
        noms_fichiers = {f["chemin"]: f for f in fichiers}

        lien_regex = re.compile(r"\[\[(.*?)\]\]|\[.*?\]\((.*?\.md)\)")

        for f in fichiers:
            chemin = f["chemin"]
            nom_affichage = f["nom"].replace(".md", "")
            noeuds.append({
                "id": chemin,
                "label": nom_affichage,
                "group": f["dossier"]
            })

            full_path = os.path.join(dossier_wiki, chemin)
            try:
                with open(full_path, "r", encoding="utf-8") as file:
                    texte = file.read()
                    matches = lien_regex.findall(texte)
                    for wiki_link, md_link in matches:
                        cible = wiki_link or md_link
                        if "|" in cible:
                            cible = cible.split("|")[0]
                        cible_clean = cible.strip()
                        if not cible_clean.endswith(".md"):
                            cible_md = cible_clean + ".md"
                        else:
                            cible_md = cible_clean

                        for cand in noms_fichiers.keys():
                            if (cand == cible_md or 
                                cand.endswith("/" + cible_md) or 
                                cand == cible_clean or 
                                cand.endswith("/" + cible_clean) or
                                cand.replace(".md", "").endswith(cible_clean.replace(".md", ""))):
                                liens.append({"from": chemin, "to": cand})
                                break
            except Exception:
                pass

        return {"nodes": noeuds, "edges": liens}


# ==============================================================================
# POINT D'ENTRÉE ET DÉMARRAGE DU SERVEUR
# ==============================================================================

def demarrer(port: int = PORT_DEFAUT) -> None:
    """Démarre le serveur HTTP multi-projets et écoute sur l'ensemble des interfaces réseau.

    Args:
        port (int): Port d'écoute du serveur HTTP (défaut : 8000).
    """
    server_address = ("0.0.0.0", port)
    httpd = HTTPServer(server_address, WikiHandler)

    ip_locale = "127.0.0.1"
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        ip_locale = s.getsockname()[0]
        s.close()
    except Exception:
        pass

    projets_connus = list(REGISTRE_PROJETS.keys())
    projet_defaut = projets_connus[0] if projets_connus else "wikiK"

    print("=" * 68)
    print(" 🚀 WikiLLM Serveur Multi-Projets Démarré !")
    print(f" 📂 Projets enregistrés : {', '.join(projets_connus)}")
    print("-" * 68)
    print(f" 🌐 Accès Hub      : http://localhost:{port}/")
    print(f" 🌐 Accès Réseau   : http://{ip_locale}:{port}/")
    print(f" 🌐 Exemple Projet : http://{ip_locale}:{port}/{projet_defaut}/index")
    print("=" * 68)
    print(" (Appuyez sur Ctrl+C pour arrêter le serveur)\n")

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nArrêt du serveur.")
        httpd.server_close()


if __name__ == "__main__":
    port_arg = PORT_DEFAUT
    if len(sys.argv) > 1:
        for a in sys.argv[1:]:
            if a.isdigit():
                port_arg = int(a)
            elif os.path.isdir(a):
                p_abs = normaliser_chemin(a)
                p_nom = os.path.basename(p_abs.rstrip(r"\/"))
                REGISTRE_PROJETS[p_nom] = {"nom": p_nom, "chemin": p_abs}
                sauvegarder_projets_enregistres(REGISTRE_PROJETS)
    demarrer(port_arg)
