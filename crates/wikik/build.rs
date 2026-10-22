//! Garantit que `front/dist` existe à la compilation : sans Node, le binaire embarque une page
//! qui explique comment construire le front (`make front`), au lieu d'échouer.

/// Le binaire embarque `front/dist` : sans front compilé, une page d'attente est créée pour que `cargo build`
/// passe quand même, et elle dit comment compiler le front.
fn main() {
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../front/dist");
    println!("cargo:rerun-if-changed={}", dist.display());
    if !dist.join("index.html").exists() {
        std::fs::create_dir_all(&dist).expect("création de front/dist");
        std::fs::write(
            dist.join("index.html"),
            "<!doctype html><meta charset=utf-8><title>wikik</title>\
             <p>Front non compilé : lancer <code>make front</code> puis recompiler wikik.</p>",
        )
        .expect("écriture de front/dist/index.html");
    }
}
