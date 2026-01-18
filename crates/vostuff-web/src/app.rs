use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::home::HomePage;
use crate::pages::login::LoginPage;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/style/main.css"/>
        <Title text="VOStuff"/>

        <Router>
            <Routes>
                <Route path="/" view=HomePage/>
                <Route path="/login" view=LoginPage/>
            </Routes>
        </Router>
    }
}
