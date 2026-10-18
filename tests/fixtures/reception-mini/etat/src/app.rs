/// Écran affiché.
pub enum Page {
    Edition,
    Bibliotheque,
}

/// Navigation.
#[component]
pub fn Nav() -> impl IntoView {
    view! {
        {lien(Page::Bibliotheque, "Mes documents")}
        <div class:hidden=move || page.get() != Page::Edition>
            <EditeurPrincipal/>
        </div>
    }
}

/// Éditeur.
#[component]
pub fn EditeurPrincipal() -> impl IntoView {
    view! { <p/> }
}

/// Bibliothèque.
#[component]
pub fn BibliothequePage() -> impl IntoView {
    view! { <p/> }
}
