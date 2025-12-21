use leptos::*;
use leptos_router::*;

use crate::server_fns::auth::logout;

#[component]
pub fn Header(
    #[prop(into)] username: String,
    #[prop(into)] org_name: String,
) -> impl IntoView {
    let navigate = use_navigate();

    let handle_logout = create_action(move |_: &()| {
        let nav = navigate.clone();
        async move {
            match logout().await {
                Ok(_) => {
                    // Redirect to login page
                    nav("/login", NavigateOptions::default());
                }
                Err(e) => {
                    // Log error but still redirect
                    tracing::error!("Logout error: {}", e);
                    nav("/login", NavigateOptions::default());
                }
            }
        }
    });

    view! {
        <header class="header">
            <div class="header-content">
                <div class="header-title">
                    "VOStuff - " {org_name}
                </div>
                <div class="header-right">
                    <span class="user-name">{username}</span>
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| {
                            handle_logout.dispatch(());
                        }
                    >

                        "Logout"
                    </button>
                </div>
            </div>
        </header>
    }
}
