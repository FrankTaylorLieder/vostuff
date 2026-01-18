use leptos::*;
use leptos_router::*;
use std::collections::HashMap;

use crate::components::header::Header;
use crate::components::items_table::ItemsTable;
use crate::components::pagination::Pagination;
use crate::server_fns::auth::{UserInfo, get_current_user};
use crate::server_fns::items::{get_items, get_locations};

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
                                view! { <AuthenticatedHome user_info=user_info/> }.into_view()
                            }
                            Ok(None) | Err(_) => {
                                // Not authenticated or error - redirect to login
                                view! { <Redirect path="/login"/> }.into_view()
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn AuthenticatedHome(user_info: UserInfo) -> impl IntoView {
    let org_id = user_info.organization.id;
    let (page, set_page) = create_signal(1i64);
    let (per_page, set_per_page) = create_signal(25i64);

    // Fetch locations once (they don't paginate)
    let locations_resource = create_resource(
        move || org_id,
        |org_id| async move { get_locations(org_id).await },
    );

    // Fetch items with pagination
    let items_resource = create_resource(
        move || (org_id, page.get(), per_page.get()),
        |(org_id, page, per_page)| async move { get_items(org_id, page, per_page).await },
    );

    view! {
        <div>
            <Header
                username=user_info.name.clone()
                org_name=user_info.organization.name.clone()
            />
            <div class="container">
                <h1>"Items"</h1>

                <Suspense fallback=move || {
                    view! { <div class="loading">"Loading items..."</div> }
                }>
                    {move || {
                        let locations_result = locations_resource.get();
                        let items_result = items_resource.get();
                        match (locations_result, items_result) {
                            (Some(Ok(locations)), Some(Ok(paginated))) => {
                                let location_map: HashMap<uuid::Uuid, String> = locations
                                    .iter()
                                    .map(|loc| (loc.id, loc.name.clone()))
                                    .collect();
                                if paginated.items.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <h3>"No items found"</h3>
                                            <p>"Start by adding your first item to this organization."</p>
                                        </div>
                                    }
                                        .into_view()
                                } else {
                                    view! {
                                        <ItemsTable items=paginated.items.clone() locations=location_map/>
                                        <Pagination
                                            current_page=page
                                            total_pages=paginated.total_pages
                                            total_items=paginated.total
                                            per_page=per_page
                                            set_page=set_page
                                            set_per_page=set_per_page
                                        />
                                    }
                                        .into_view()
                                }
                            }
                            (Some(Err(e)), _) | (_, Some(Err(e))) => {
                                view! {
                                    <div class="error">
                                        {format!("Error loading data: {}", e)}
                                    </div>
                                }
                                    .into_view()
                            }
                            _ => {
                                view! { <div class="loading">"Loading..."</div> }.into_view()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
