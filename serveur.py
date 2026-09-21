#!/usr/bin/env python3
"""
Serveur Web léger pour le Wiki de Projet (LLM-Wiki).
Fonctionne avec la bibliothèque standard Python (zéro dépendance externe à installer).
"""

import os
import json
import urllib.parse
from http.server import HTTPServer, SimpleHTTPRequestHandler
import webbrowser
import sys

PORT = 8000
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
WIKI_DIR = os.path.join(BASE_DIR, "wiki")

# Création du dossier wiki s'il n'existe pas
os.makedirs(WIKI_DIR, exist_ok=True)
for sub in ["decisions", "sessions", "architecture", "taches"]:
    os.makedirs(os.path.join(WIKI_DIR, sub), exist_ok=True)

class WikiHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=BASE_DIR, **kwargs)

    def do_GET(self):
        parsed_url = urllib.parse.urlparse(self.path)
        path = parsed_url.path

        # Route API : Liste de tous les fichiers du wiki
        if path == "/api/fichiers":
            self.send_json(self.lister_fichiers())
            return

        # Route API : Obtenir le contenu d'un fichier Markdown
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

        # Route API : Obtenir les données du graphe (liens entre pages)
        elif path == "/api/graphe":
            self.send_json(self.generer_graphe())
            return

        # Fichier par défaut : index.html
        if path == "/" or path == "":
            self.path = "/index.html"

        return super().do_GET()

    def do_POST(self):
        parsed_url = urllib.parse.urlparse(self.path)
        path = parsed_url.path

        # Route API : Sauvegarder ou créer un fichier
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
                        cible_clean = cible.strip()
                        if not cible_clean.endswith(".md"):
                            cible_md = cible_clean + ".md"
                        else:
                            cible_md = cible_clean

                        # Recherche de la cible
                        for cand in noms_fichiers.keys():
                            if cand == cible_md or cand.endswith("/" + cible_md) or cand.endswith("/" + cible_clean):
                                liens.append({"from": chemin, "to": cand})
                                break
            except Exception:
                pass

        return {"nodes": noeuds, "edges": liens}

def demarrer(port=PORT):
    server_address = ("", port)
    httpd = HTTPServer(server_address, WikiHandler)
    url = f"http://localhost:{port}"
    print("=" * 60)
    print(f" 🚀 WikiLLM Viewer & Éditeur démarré avec succès !")
    print(f" 🌐 Ouvrez votre navigateur sur : {url}")
    print(f" 📁 Répertoire des notes : {WIKI_DIR}")
    print("=" * 60)
    print(" (Appuyez sur Ctrl+C pour arrêter le serveur)")

    # Essaye d'ouvrir le navigateur si exécuté avec interface
    try:
        if "--open" in sys.argv:
            webbrowser.open(url)
    except Exception:
        pass

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nArrêt du serveur.")
        httpd.server_close()

if __name__ == "__main__":
    port_arg = PORT
    if len(sys.argv) > 1 and sys.argv[1].isdigit():
        port_arg = int(sys.argv[1])
    demarrer(port_arg)
