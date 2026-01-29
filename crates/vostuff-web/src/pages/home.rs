use leptos::*;
use leptos_router::*;
use std::collections::{HashMap, HashSet};

use crate::components::filter_dropdown::{
    FilterBar, FilterDropdown, FilterOption, FilterSearchInput,
};
use crate::components::header::Header;
use crate::components::items_table::ItemsTable;
use crate::components::pagination::Pagination;
use crate::server_fns::auth::{UserInfo, get_current_user};
use crate::server_fns::items::{ItemFilters, ItemState, ItemType, get_items, get_locations};

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

    // Pagination state
    let (page, set_page) = create_signal(1i64);
    let (per_page, set_per_page) = create_signal(25i64);

    // Filter state
    let (selected_types, set_selected_types) = create_signal::<HashSet<String>>(HashSet::new());
    let (selected_states, set_selected_states) = create_signal::<HashSet<String>>(HashSet::new());
    let (selected_locations, set_selected_locations) =
        create_signal::<HashSet<String>>(HashSet::new());
    let (search_input, set_search_input) = create_signal(String::new());
    let (search_text, set_search_text) = create_signal(String::new());

    // Sort state
    let (sort_by, set_sort_by) = create_signal("name".to_string());
    let (sort_order, set_sort_order) = create_signal("asc".to_string());

    // Refresh counter to trigger items refetch after edits
    let (refresh_counter, set_refresh_counter) = create_signal(0u32);

    // Expanded row state (owned here so it persists across refetches)
    let (expanded_row, set_expanded_row) = create_signal::<Option<uuid::Uuid>>(None);

    // Reset to page 1 when filters change
    create_effect(move |_| {
        let _ = selected_types.get();
        let _ = selected_states.get();
        let _ = selected_locations.get();
        let _ = search_text.get();
        set_page.set(1);
    });

    // Fetch locations once (they don't paginate)
    let locations_resource = create_resource(
        move || org_id,
        |org_id| async move { get_locations(org_id).await },
    );

    // Fetch items with pagination and filters
    // Convert HashSets to sorted Vecs for stable comparison in resource source
    let items_resource = create_resource(
        move || {
            let mut types: Vec<String> = selected_types.get().into_iter().collect();
            types.sort();
            let mut states: Vec<String> = selected_states.get().into_iter().collect();
            states.sort();
            let mut locations: Vec<String> = selected_locations.get().into_iter().collect();
            locations.sort();
            let search = search_text.get();
            let sb = sort_by.get();
            let so = sort_order.get();
            let rc = refresh_counter.get();
            (
                org_id,
                page.get(),
                per_page.get(),
                types,
                states,
                locations,
                search,
                sb,
                so,
                rc,
            )
        },
        move |(org_id, page, per_page, types, states, locations, search, sb, so, _rc)| {
            // Build filters from the source values
            let location_ids: Vec<uuid::Uuid> = locations
                .iter()
                .filter_map(|s| uuid::Uuid::parse_str(s).ok())
                .collect();

            let search_query = if search.is_empty() {
                None
            } else {
                Some(search)
            };

            let sort_by_opt = Some(sb);
            let sort_order_opt = Some(so);

            let filters = if types.is_empty()
                && states.is_empty()
                && location_ids.is_empty()
                && search_query.is_none()
                && sort_by_opt.as_deref() == Some("name")
                && sort_order_opt.as_deref() == Some("asc")
            {
                None
            } else {
                Some(ItemFilters {
                    item_types: types,
                    states,
                    location_ids,
                    search_query,
                    sort_by: sort_by_opt,
                    sort_order: sort_order_opt,
                })
            };

            async move { get_items(org_id, page, per_page, filters).await }
        },
    );

    // Build filter options for types (stored for reuse in reactive context)
    let type_options = store_value(
        ItemType::all()
            .into_iter()
            .map(|t| FilterOption {
                value: t.api_value().to_string(),
                label: t.display_name().to_string(),
            })
            .collect::<Vec<_>>(),
    );

    // Build filter options for states (stored for reuse in reactive context)
    let state_options = store_value(
        ItemState::all()
            .into_iter()
            .map(|s| FilterOption {
                value: s.api_value().to_string(),
                label: s.display_name().to_string(),
            })
            .collect::<Vec<_>>(),
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
                    view! { <div class="loading">"Loading..."</div> }
                }>
                    {move || {
                        let locations_result = locations_resource.get();
                        let items_result = items_resource.get();
                        match (locations_result, items_result) {
                            (Some(Ok(locations)), Some(Ok(paginated))) => {
                                // Build location map for table display
                                let location_map: HashMap<uuid::Uuid, String> = locations
                                    .iter()
                                    .map(|loc| (loc.id, loc.name.clone()))
                                    .collect();
                                // Build location options for filter
                                let location_options: Vec<FilterOption> = locations
                                    .iter()
                                    .map(|loc| FilterOption {
                                        value: loc.id.to_string(),
                                        label: loc.name.clone(),
                                    })
                                    .collect();
                                let has_filters = !selected_types.get().is_empty()
                                    || !selected_states.get().is_empty()
                                    || !selected_locations.get().is_empty()
                                    || !search_text.get().is_empty();
                                view! {
                                    <FilterBar>
                                        <FilterSearchInput
                                            value=search_input
                                            set_value=set_search_input
                                            set_committed=set_search_text
                                        />
                                        <FilterDropdown
                                            label="Type"
                                            options=type_options.get_value()
                                            selected=selected_types
                                            set_selected=set_selected_types
                                        />
                                        <FilterDropdown
                                            label="State"
                                            options=state_options.get_value()
                                            selected=selected_states
                                            set_selected=set_selected_states
                                        />
                                        <FilterDropdown
                                            label="Location"
                                            options=location_options
                                            selected=selected_locations
                                            set_selected=set_selected_locations
                                        />
                                        <Show when=move || has_filters fallback=|| ()>
                                            <button
                                                class="filter-clear-btn"
                                                on:click=move |_| {
                                                    set_selected_types.set(std::collections::HashSet::new());
                                                    set_selected_states.set(std::collections::HashSet::new());
                                                    set_selected_locations.set(std::collections::HashSet::new());
                                                    set_search_input.set(String::new());
                                                    set_search_text.set(String::new());
                                                }
                                            >
                                                "Clear Filters"
                                            </button>
                                        </Show>
                                    </FilterBar>

                                    {if paginated.items.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <h3>"No items found"</h3>
                                                <p>
                                                    {if has_filters {
                                                        "No items match the current filters. Try adjusting your filter criteria."
                                                    } else {
                                                        "Start by adding your first item to this organization."
                                                    }}
                                                </p>
                                            </div>
                                        }
                                            .into_view()
                                    } else {
                                        view! {
                                            <ItemsTable
                                                items=paginated.items.clone()
                                                locations=location_map
                                                locations_list=locations.clone()
                                                search_query=search_text.get()
                                                sort_by=sort_by.get()
                                                sort_order=sort_order.get()
                                                set_sort_by=set_sort_by
                                                set_sort_order=set_sort_order
                                                on_item_updated=Callback::new(move |()| set_refresh_counter.update(|c| *c += 1))
                                                expanded_row=expanded_row
                                                set_expanded_row=set_expanded_row
                                                org_id=org_id
                                            />
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
                                    }}
                                }
                                    .into_view()
                            }
                            (Some(Err(e)), _) | (_, Some(Err(e))) => {
                                view! {
                                    <div class="error">{format!("Error loading data: {}", e)}</div>
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
