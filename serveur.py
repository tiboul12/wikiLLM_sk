#!/usr/bin/env python3
"""
Serveur Web Universel Multi-Projets pour Wiki de Projet (LLM-Wiki).
- Scanner de projets (Linux / Mac / Windows).
- Routage d'URL propre (ex: http://10.0.0.120:8000/nom_projet/note).
- Zéro dépendance externe (Python 3 standard uniquement).
"""

import os
import sys
import json
import socket
import urllib.parse
from pathlib import Path
from http.server import HTTPServer, SimpleHTTPRequestHandler

PORT_DEFAUT = 8000
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
CONFIG_FILE = os.path.join(SCRIPT_DIR, ".wikillm_projets.json")

def charger_projets_enregistres():
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

def sauvegarder_projets_enregistres(projets):
    try:
        with open(CONFIG_FILE, "w", encoding="utf-8") as f:
            json.dump(projets, f, indent=2, ensure_ascii=False)
    except Exception as e:
        print(f"Erreur sauvegarde config projets: {e}")

REGISTRE_PROJETS = charger_projets_enregistres()

def normaliser_chemin(chemin_brut):
    """Gère l'expansion ~ sous Linux/Mac et les chemins Windows C:\..."""
    if not chemin_brut:
        return ""
    chemin = chemin_brut.strip()
    if chemin.startswith("~"):
        chemin = os.path.expanduser(chemin)
    return os.path.abspath(chemin)

def scanner_dossier_racine(chemin_racine):
    """Scanne un dossier pour trouver tous les sous-dossiers et détecter les dossiers wiki."""
    racine = normaliser_chemin(chemin_racine)
    resultats = []

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

def compter_notes(dossier_wiki):
    total = 0
    if not os.path.isdir(dossier_wiki):
        return 0
    for _, _, fichiers in os.walk(dossier_wiki):
        total += sum(1 for f in fichiers if f.endswith(".md"))
    return total

def initialiser_wiki_projet(chemin_projet):
    """Initialise la structure wiki standard dans un projet."""
    chemin = normaliser_chemin(chemin_projet)
    nom_projet = os.path.basename(chemin.rstrip(r"\/"))
    dossier_wiki = os.path.join(chemin, "wiki")

    os.makedirs(dossier_wiki, exist_ok=True)
    for sous in ["decisions", "sessions", "architecture", "taches"]:
        os.makedirs(os.path.join(dossier_wiki, sous), exist_ok=True)

    index_path = os.path.join(dossier_wiki, "index.md")
    if not os.path.exists(index_path):
        with open(index_path, "w", encoding="utf-8") as f:
            f.write(f"""# 📖 Wiki de {nom_projet}

Bienvenue dans la documentation vivante du projet **{nom_projet}**.

## 🗂️ Sommaire Général
- **[[architecture/vue-ensemble|Architecture & Spécifications]]**
- **[[decisions/001-initialisation|Registre des Décisions (ADR)]]**
- **[[sessions/session-initiale|Journal des Sessions]]**
- **[[taches/backlog|Feuille de Route & Tâches]]**
""")
    return dossier_wiki

class WikiHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=SCRIPT_DIR, **kwargs)

    def obtenir_projet_actif(self, nom_projet_ou_chemin=None):
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

    def do_GET(self):
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
        # Toutes les URLs comme /wikiK ou /wikiK/taches/backlog renvoient index.html
        if not "." in path or path.endswith(".html"):
            self.path = "/index.html"

        return super().do_GET()

    def do_POST(self):
        parsed_url = urllib.parse.urlparse(self.path)
        path = parsed_url.path.strip("/")
        params = urllib.parse.parse_qs(parsed_url.query)

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

    def lire_corps_json(self):
        content_length = int(self.headers.get("Content-Length", 0))
        if content_length == 0:
            return None
        body = self.rfile.read(content_length).decode("utf-8")
        try:
            return json.loads(body)
        except json.JSONDecodeError:
            return None

    def send_json(self, data):
        reponse = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(reponse)))
        self.end_headers()
        self.wfile.write(reponse)

    def lister_fichiers(self, dossier_wiki):
        arborescence = []
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

    def generer_graphe(self, dossier_wiki):
        noeuds = []
        liens = []
        fichiers = self.lister_fichiers(dossier_wiki)
        noms_fichiers = {f["chemin"]: f for f in fichiers}

        import re
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

def demarrer(port=PORT_DEFAUT):
    server_address = ("0.0.0.0", port)
    httpd = HTTPServer(server_address, WikiHandler)
    
    ip_locale = "10.0.0.120"
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
    print(f" 🚀 WikiLLM Serveur Multi-Projets Démarré !")
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
