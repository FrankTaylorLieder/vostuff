use leptos::*;
use leptos_router::*;

use crate::components::fields_manager::FieldsManager;
use crate::components::header::Header;
use crate::components::kinds_manager::KindsManager;
use crate::server_fns::auth::{UserInfo, get_current_user};

#[derive(Clone, PartialEq)]
enum Tab {
    Kinds,
    Fields,
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let user_resource = create_resource(|| (), |_| async move { get_current_user().await });

    view! {
        <div>
            <Suspense fallback=move || view! { <div class="container">"Loading..."</div> }>
                {move || {
                    user_resource
                        .get()
                        .map(|result| match result {
                            Ok(Some(user_info)) => {
                                view! { <AuthenticatedSettings user_info=user_info/> }.into_view()
                            }
                            Ok(None) | Err(_) => {
                                view! { <Redirect path="/login"/> }.into_view()
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn AuthenticatedSettings(user_info: UserInfo) -> impl IntoView {
    let org_id = user_info.organization.id;
    let (active_tab, set_active_tab) = create_signal(Tab::Kinds);

    view! {
        <div>
            <Header
                username=user_info.name.clone()
                org_name=user_info.organization.name.clone()
            />
            <div class="container">
                <div class="page-header">
                    <h1>"Settings"</h1>
                </div>
                <div class="tab-bar">
                    <button
                        class=move || {
                            if active_tab.get() == Tab::Kinds { "tab-btn active" } else { "tab-btn" }
                        }
                        on:click=move |_| set_active_tab.set(Tab::Kinds)
                    >
                        "Kinds"
                    </button>
                    <button
                        class=move || {
                            if active_tab.get() == Tab::Fields {
                                "tab-btn active"
                            } else {
                                "tab-btn"
                            }
                        }
                        on:click=move |_| set_active_tab.set(Tab::Fields)
                    >
                        "Fields"
                    </button>
                </div>
                <Show when=move || active_tab.get() == Tab::Kinds fallback=|| ()>
                    <KindsManager org_id=org_id/>
                </Show>
                <Show when=move || active_tab.get() == Tab::Fields fallback=|| ()>
                    <FieldsManager org_id=org_id/>
                </Show>
            </div>
        </div>
    }
}
