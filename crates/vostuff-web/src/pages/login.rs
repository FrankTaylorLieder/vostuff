use leptos::*;
use leptos_router::*;
use uuid::Uuid;

use crate::server_fns::auth::{
    LoginResponse, OrgSelectionResponse, OrganizationWithRoles, login, select_organization,
};

#[derive(Clone, Debug)]
enum LoginState {
    Initial,
    SelectingOrg(OrgSelectionResponse),
    Success(LoginResponse),
    Error(String),
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (identity, set_identity) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (login_state, set_login_state) = create_signal(LoginState::Initial);
    let (is_loading, set_is_loading) = create_signal(false);

    let navigate = use_navigate();
    let nav1 = navigate.clone();
    let nav2 = navigate.clone();

    // Handle login form submission
    let handle_login = create_action(move |_: &()| {
        let identity_val = identity.get();
        let password_val = password.get();
        let nav = nav1.clone();

        async move {
            set_is_loading.set(true);

            match login(identity_val, password_val, None).await {
                Ok(Ok(login_resp)) => {
                    // Direct login success - redirect to home
                    set_login_state.set(LoginState::Success(login_resp));
                    set_is_loading.set(false);
                    nav("/", NavigateOptions::default());
                }
                Ok(Err(org_selection)) => {
                    // Need to select organization
                    set_login_state.set(LoginState::SelectingOrg(org_selection));
                    set_is_loading.set(false);
                }
                Err(e) => {
                    set_login_state.set(LoginState::Error(e.to_string()));
                    set_is_loading.set(false);
                }
            }
        }
    });

    // Handle organization selection
    let handle_org_select = create_action(move |(follow_on_token, org_id): &(String, Uuid)| {
        let token = follow_on_token.clone();
        let org = *org_id;
        let nav = nav2.clone();

        async move {
            set_is_loading.set(true);

            match select_organization(token, org).await {
                Ok(login_resp) => {
                    set_login_state.set(LoginState::Success(login_resp));
                    set_is_loading.set(false);
                    nav("/", NavigateOptions::default());
                }
                Err(e) => {
                    set_login_state.set(LoginState::Error(e.to_string()));
                    set_is_loading.set(false);
                }
            }
        }
    });

    view! {
        <div class="container">
            {move || match login_state.get() {
                LoginState::Initial | LoginState::Error(_) | LoginState::Success(_) => {
                    view! {
                        <div class="form">
                            <h1 class="form-title">"VOStuff Login"</h1>

                            {move || {
                                if let LoginState::Error(err) = login_state.get() {
                                    view! { <div class="error">{err}</div> }.into_view()
                                } else {
                                    view! { <></> }.into_view()
                                }
                            }}

                            <form on:submit=move |ev| {
                                ev.prevent_default();
                                handle_login.dispatch(());
                            }>
                                <div class="form-group">
                                    <label class="form-label">"Email"</label>
                                    <input
                                        type="email"
                                        class="form-input"
                                        placeholder="user@example.com"
                                        prop:value=identity
                                        on:input=move |ev| {
                                            set_identity.set(event_target_value(&ev));
                                        }

                                        required
                                    />
                                </div>

                                <div class="form-group">
                                    <label class="form-label">"Password"</label>
                                    <input
                                        type="password"
                                        class="form-input"
                                        placeholder="Enter your password"
                                        prop:value=password
                                        on:input=move |ev| {
                                            set_password.set(event_target_value(&ev));
                                        }

                                        required
                                    />
                                </div>

                                <button
                                    type="submit"
                                    class="btn btn-primary"
                                    disabled=move || is_loading.get()
                                >
                                    {move || if is_loading.get() { "Logging in..." } else { "Login" }}
                                </button>
                            </form>
                        </div>
                    }
                        .into_view()
                }
                LoginState::SelectingOrg(org_selection) => {
                    view! {
                        <div class="form">
                            <h1 class="form-title">"Select Organization"</h1>
                            <p class="text-center mb-16">
                                "You belong to multiple organizations. Please select one:"
                            </p>

                            <ul class="org-list">
                                {org_selection
                                    .organizations
                                    .iter()
                                    .map(|org: &OrganizationWithRoles| {
                                        let org_clone = org.clone();
                                        let token = org_selection.follow_on_token.clone();
                                        view! {
                                            <li
                                                class="org-item"
                                                on:click=move |_| {
                                                    handle_org_select
                                                        .dispatch((token.clone(), org_clone.id));
                                                }
                                            >

                                                <div class="org-name">{&org.name}</div>
                                                <div class="org-roles">
                                                    "Roles: " {org.roles.join(", ")}
                                                </div>
                                            </li>
                                        }
                                    })
                                    .collect_view()}

                            </ul>
                        </div>
                    }
                        .into_view()
                }
            }}

        </div>
    }
}
