/// Application Leptos.
#[component]
pub fn App() -> impl IntoView {
    view! { <Router><Routes><Route path=path!("/profil") view=Profil/></Routes></Router> }
}

/// Profil de l'utilisateur.
#[component]
pub fn Profil() -> impl IntoView {
    view! { <p>"profil"</p> }
}
