#!/usr/bin/env python3
"""
Serveur Web léger pour Wiki de Projet (LLM-Wiki).
Fonctionne avec la bibliothèque standard Python (0 dépendance externe à installer).

Usage:
    python3 serveur.py [chemin_du_projet] [port]
    
Exemples:
    python3 serveur.py                               # Projet courant, port 8000
    python3 serveur.py /home/tibal12/projets/siteA   # Projet siteA, port 8000
    python3 serveur.py /home/tibal12/projets/siteB 8080
"""

import os
import sys
import json
import socket
import urllib.parse
from http.server import HTTPServer, SimpleHTTPRequestHandler

PORT_DEFAUT = 8000
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# Analyse des arguments CLI
def analyser_arguments():
    repertoire_projet = os.getcwd()
    port = PORT_DEFAUT

    args = [a for a in sys.argv[1:] if not a.startswith("-")]

    for arg in args:
        if arg.isdigit():
            port = int(arg)
        elif os.path.isdir(arg) or not os.path.exists(arg):
            repertoire_projet = os.path.abspath(arg)

    return repertoire_projet, port

REPERTOIRE_PROJET, PORT = analyser_arguments()
WIKI_DIR = os.path.join(REPERTOIRE_PROJET, "wiki")
NOM_PROJET = os.path.basename(REPERTOIRE_PROJET.rstrip("/"))

# Initialisation automatique du wiki dans le projet s'il n'existe pas encore
def initialiser_wiki_si_absent():
    os.makedirs(WIKI_DIR, exist_ok=True)
    for sous_dossier in ["decisions", "sessions", "architecture", "taches"]:
        os.makedirs(os.path.join(WIKI_DIR, sous_dossier), exist_ok=True)

    index_path = os.path.join(WIKI_DIR, "index.md")
    if not os.path.exists(index_path):
        with open(index_path, "w", encoding="utf-8") as f:
            f.write(f"""# 📖 Wiki de {NOM_PROJET}

Bienvenue dans la documentation vivante du projet **{NOM_PROJET}**.

## 🗂️ Sommaire Général
- **[[architecture/vue-ensemble|Architecture & Spécifications]]**
- **[[decisions/001-initialisation|Registre des Décisions (ADR)]]**
- **[[sessions/session-initiale|Journal des Sessions]]**
- **[[taches/backlog|Feuille de Route & Tâches]]**
""")

initialiser_wiki_si_absent()

class WikiHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        # Sert les fichiers statiques (index.html, etc.) depuis le dossier du script
        super().__init__(*args, directory=SCRIPT_DIR, **kwargs)

    def do_GET(self):
        parsed_url = urllib.parse.urlparse(self.path)
        path = parsed_url.path

        # Route API : Informations sur le projet actif
        if path == "/api/projet_info":
            self.send_json({
                "nom_projet": NOM_PROJET,
                "repertoire": REPERTOIRE_PROJET,
                "dossier_wiki": WIKI_DIR
            })
            return

        # Route API : Liste des fichiers du wiki
        elif path == "/api/fichiers":
            self.send_json(self.lister_fichiers())
            return

        # Route API : Contenu d'un fichier Markdown
        elif path == "/api/fichier":
            params = urllib.parse.parse_qs(parsed_url.query)
            rel_path = params.get("chemin", [""])[0]
            if not rel_path:
                self.send_error(400, "Chemin manquant")
                return
            
            full_path = os.path.abspath(os.path.join(WIKI_DIR, rel_path))
            if not full_path.startswith(WIKI_DIR) or not os.path.exists(full_path):
                self.send_error(404, "Fichier non trouvé")
                return

            try:
                with open(full_path, "r", encoding="utf-8") as f:
                    contenu = f.read()
                self.send_json({"chemin": rel_path, "contenu": contenu})
            except Exception as e:
                self.send_error(500, f"Erreur lecture: {str(e)}")
            return

        # Route API : Données du graphe relationnel
        elif path == "/api/graphe":
            self.send_json(self.generer_graphe())
            return

        # Page d'accueil
        if path == "/" or path == "":
            self.path = "/index.html"

        return super().do_GET()

    def do_POST(self):
        parsed_url = urllib.parse.urlparse(self.path)
        path = parsed_url.path

        # Route API : Sauvegarder un fichier
        if path == "/api/sauvegarder":
            data = self.lire_corps_json()
            if not data or "chemin" not in data or "contenu" not in data:
                self.send_error(400, "Données invalides")
                return

            rel_path = data["chemin"].strip("/")
            if not rel_path.endswith(".md"):
                rel_path += ".md"

            full_path = os.path.abspath(os.path.join(WIKI_DIR, rel_path))
            if not full_path.startswith(WIKI_DIR):
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

        # Route API : Supprimer un fichier
        elif path == "/api/supprimer":
            data = self.lire_corps_json()
            if not data or "chemin" not in data:
                self.send_error(400, "Chemin manquant")
                return

            rel_path = data["chemin"].strip("/")
            full_path = os.path.abspath(os.path.join(WIKI_DIR, rel_path))
            if not full_path.startswith(WIKI_DIR) or not os.path.exists(full_path):
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

    def lister_fichiers(self):
        arborescence = []
        for racine, dossiers, fichiers in os.walk(WIKI_DIR):
            for f in sorted(fichiers):
                if f.endswith(".md"):
                    chemin_complet = os.path.join(racine, f)
                    chemin_relatif = os.path.relpath(chemin_complet, WIKI_DIR)
                    dossier = os.path.dirname(chemin_relatif)
                    arborescence.append({
                        "nom": f,
                        "chemin": chemin_relatif,
                        "dossier": dossier if dossier else "Racine",
                        "taille": os.path.getsize(chemin_complet)
                    })
        return arborescence

    def generer_graphe(self):
        noeuds = []
        liens = []
        fichiers = self.lister_fichiers()
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

            full_path = os.path.join(WIKI_DIR, chemin)
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
                            if cand == cible_md or cand.endswith("/" + cible_md) or cand.endswith("/" + cible_clean):
                                liens.append({"from": chemin, "to": cand})
                                break
            except Exception:
                pass

        return {"nodes": noeuds, "edges": liens}

def demarrer(port=PORT):
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

    print("=" * 65)
    print(f" 🚀 WikiLLM - Projet : {NOM_PROJET}")
    print(f" 📁 Dossier racine   : {REPERTOIRE_PROJET}")
    print(f" 📂 Emplacement wiki : {WIKI_DIR}")
    print("-" * 65)
    print(f" 🌐 Accès local      : http://localhost:{port}")
    print(f" 🌐 Accès réseau     : http://{ip_locale}:{port}")
    print("=" * 65)
    print(" (Appuyez sur Ctrl+C pour arrêter le serveur)\n")

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nArrêt du serveur.")
        httpd.server_close()

if __name__ == "__main__":
    demarrer(PORT)
