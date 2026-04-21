use leptos::*;
use leptos_router::*;

use crate::server_fns::auth::logout;

#[component]
pub fn Header(#[prop(into)] username: String, #[prop(into)] org_name: String) -> impl IntoView {
    let navigate = use_navigate();
    let navigate2 = navigate.clone();

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
                    <a href="/" style="color: inherit; text-decoration: none; display: inline-flex; align-items: center; gap: 8px;">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="18"
                            height="18"
                            viewBox="0 0 24 24"
                            fill="currentColor"
                            style="flex-shrink: 0; opacity: 0.7;"
                        >
                            <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>
                        </svg>
                        "VOStuff - " {org_name}
                    </a>
                </div>
                <div class="header-right">
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| {
                            navigate2("/settings", NavigateOptions::default());
                        }
                    >
                        "Settings"
                    </button>
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
