use leptos::*;
use leptos_router::*;

use crate::components::header::Header;
use crate::server_fns::auth::get_current_user;

#[component]
pub fn HomePage() -> impl IntoView {
    // Fetch current user on component mount
    let user_resource = create_resource(|| (), |_| async move { get_current_user().await });

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
                            Ok(None) | Err(_) => {
                                // Not authenticated or error - redirect to login
                                view! { <Redirect path="/login"/> }
                                    .into_view()
                            }
                        })
                }}

            </Suspense>
        </div>
    }
}
