//! Outil de mise au point : affiche l'arbre tree-sitter d'un fichier TypeScript, avec les noms de champs.
//!
//! Sert à écrire ou corriger les règles d'extraction d'un langage (SPEC-15) :
//! `cargo run -p wikik-core --example arbre_ts -- fichier.ts`.
/// Affiche l'arbre tree-sitter (grammaire TSX) d'un fichier, pour écrire ou corriger l'extraction TypeScript.
fn main() {
    let chemin = std::env::args().nth(1).unwrap();
    let src = std::fs::read(&chemin).unwrap();
    let mut p = tree_sitter::Parser::new();
    p.set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
        .unwrap();
    let t = p.parse(&src, None).unwrap();
    fn afficher(n: tree_sitter::Node, src: &[u8], prof: usize, champ: Option<&str>) {
        let texte = if n.child_count() == 0 {
            format!(" {:?}", n.utf8_text(src).unwrap())
        } else {
            String::new()
        };
        println!(
            "{}{}{}{}",
            "  ".repeat(prof),
            champ.map(|c| format!("{c}: ")).unwrap_or_default(),
            n.kind(),
            texte
        );
        let mut c = n.walk();
        for (i, e) in n.children(&mut c).enumerate() {
            afficher(e, src, prof + 1, n.field_name_for_child(i as u32));
        }
    }
    afficher(t.root_node(), &src, 0, None);
}
