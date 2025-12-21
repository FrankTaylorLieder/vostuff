use leptos::*;
use leptos_router::*;

use crate::components::header::Header;
use crate::server_fns::auth::get_current_user;

#[component]
pub fn HomePage() -> impl IntoView {
    let navigate = use_navigate();

    // Fetch current user on component mount
    let user_resource = create_resource(
        || (),
        |_| async move {
            get_current_user().await
        },
    );

    // Effect to redirect if not authenticated
    let nav = navigate.clone();
    create_effect(move |_| {
        if let Some(result) = user_resource.get() {
            match result {
                Ok(None) => {
                    // Not authenticated - redirect to login
                    nav("/login", NavigateOptions::default());
                }
                Err(_) => {
                    // Error checking auth - redirect to login
                    nav("/login", NavigateOptions::default());
                }
                Ok(Some(_)) => {
                    // Authenticated - stay on page
                }
            }
        }
    });

    view! {
        <div>
            <Suspense fallback=move || view! { <div class="container">"Loading..."</div> }>
                {move || {
                    user_resource
                        .get()
                        .map(|result| match result {
                            Ok(Some(user_info)) => {
                                view! {
                                    <div>
                                        <Header
                                            username=user_info.name.clone()
                                            org_name=user_info.organization.name.clone()
                                        />
                                        <div class="container">
                                            <h1>"Welcome to VOStuff"</h1>
                                            <p>"This is the main page. Content coming soon..."</p>
                                        </div>
                                    </div>
                                }
                                    .into_view()
                            }
                            _ => {
                                view! { <div class="container">"Redirecting to login..."</div> }
                                    .into_view()
                            }
                        })
                }}

            </Suspense>
        </div>
    }
}
