use leptos::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::server_fns::items::{Item, ItemFullDetails, get_item_details};

fn highlight_match(text: &str, query: &str) -> View {
    if query.is_empty() {
        return text.to_string().into_view();
    }
    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut fragments: Vec<View> = Vec::new();
    let mut start = 0;
    while let Some(pos) = lower_text[start..].find(&lower_query) {
        let abs_pos = start + pos;
        if abs_pos > start {
            fragments.push(text[start..abs_pos].to_string().into_view());
        }
        let matched = &text[abs_pos..abs_pos + query.len()];
        fragments.push(
            view! { <mark class="search-highlight">{matched.to_string()}</mark> }.into_view(),
        );
        start = abs_pos + query.len();
    }
    if start < text.len() {
        fragments.push(text[start..].to_string().into_view());
    }
    fragments.collect_view()
}

#[component]
pub fn ItemsTable(
    items: Vec<Item>,
    locations: HashMap<Uuid, String>,
    #[prop(default = String::new())] search_query: String,
    #[prop(default = "name".to_string())] sort_by: String,
    #[prop(default = "asc".to_string())] sort_order: String,
    #[prop(optional)] set_sort_by: Option<WriteSignal<String>>,
    #[prop(optional)] set_sort_order: Option<WriteSignal<String>>,
    org_id: Uuid,
) -> impl IntoView {
    let (expanded_row, set_expanded_row) = create_signal::<Option<Uuid>>(None);

    let toggle_row = move |item_id: Uuid| {
        set_expanded_row.update(|current| {
            if *current == Some(item_id) {
                *current = None;
            } else {
                *current = Some(item_id);
            }
        });
    };

    let sort_by_clone = sort_by.clone();
    let sort_order_clone = sort_order.clone();

    let make_sort_handler = move |column: &'static str| {
        let sb = sort_by_clone.clone();
        let so = sort_order_clone.clone();
        move |_: web_sys::MouseEvent| {
            if let (Some(set_sb), Some(set_so)) = (set_sort_by, set_sort_order) {
                if sb == column {
                    set_so.set(if so == "asc" {
                        "desc".to_string()
                    } else {
                        "asc".to_string()
                    });
                } else {
                    set_sb.set(column.to_string());
                    set_so.set("asc".to_string());
                }
            }
        }
    };

    let sort_indicator = |column: &str| -> &'static str {
        if sort_by == column {
            if sort_order == "asc" {
                " \u{25B2}"
            } else {
                " \u{25BC}"
            }
        } else {
            ""
        }
    };

    let on_type = make_sort_handler("item_type");
    let on_name = make_sort_handler("name");
    let on_state = make_sort_handler("state");
    let on_location = make_sort_handler("location_id");

    let ind_type = sort_indicator("item_type");
    let ind_name = sort_indicator("name");
    let ind_state = sort_indicator("state");
    let ind_location = sort_indicator("location_id");

    view! {
        <table class="items-table">
            <thead>
                <tr>
                    <th class="col-type sortable-header" on:click=on_type>{format!("Type{}", ind_type)}</th>
                    <th class="col-name sortable-header" on:click=on_name>{format!("Name{}", ind_name)}</th>
                    <th class="col-state sortable-header" on:click=on_state>{format!("State{}", ind_state)}</th>
                    <th class="col-location sortable-header" on:click=on_location>{format!("Location{}", ind_location)}</th>
                </tr>
            </thead>
            <tbody>
                {items
                    .into_iter()
                    .map(|item| {
                        let item_id = item.id;
                        let location_name = item
                            .location_id
                            .and_then(|loc_id| locations.get(&loc_id).cloned())
                            .unwrap_or_else(|| "-".to_string());
                        let is_expanded = move || expanded_row.get() == Some(item_id);
                        let item_for_details = item.clone();
                        let sq = search_query.clone();
                        let sq2 = search_query.clone();
                        view! {
                            <tr
                                class="item-row"
                                class:expanded=is_expanded
                                on:click=move |_| toggle_row(item_id)
                            >
                                <td class="col-type">{item.item_type.display_name()}</td>
                                <td class="col-name">{highlight_match(&item.name, &sq)}</td>
                                <td class="col-state">
                                    <span class=format!("state-badge {}", item.state.css_class())>
                                        {item.state.display_name()}
                                    </span>
                                </td>
                                <td class="col-location">{location_name.clone()}</td>
                            </tr>
                            <Show when=is_expanded fallback=|| ()>
                                <ItemExpandedRow item=item_for_details.clone() location_name=location_name.clone() search_query=sq2.clone() org_id=org_id/>
                            </Show>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

fn render_type_details(details: &ItemFullDetails) -> View {
    if let Some(ref vd) = details.vinyl_details {
        let size = vd.size.as_ref().map(|s| s.display_name()).unwrap_or("-");
        let speed = vd.speed.as_ref().map(|s| s.display_name()).unwrap_or("-");
        let channels = vd
            .channels
            .as_ref()
            .map(|c| c.display_name())
            .unwrap_or("-");
        let disks = vd
            .disks
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());
        let media = vd
            .media_grading
            .as_ref()
            .map(|g| g.display_name())
            .unwrap_or("-");
        let sleeve = vd
            .sleeve_grading
            .as_ref()
            .map(|g| g.display_name())
            .unwrap_or("-");
        view! {
            <div class="detail-section">
                <h4>"Vinyl Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Size:"</span>
                        <span class="detail-value">{size}</span>
                    </div>
                    <div class="detail-group">
                        <span class="detail-label">"Speed:"</span>
                        <span class="detail-value">{speed}</span>
                    </div>
                    <div class="detail-group">
                        <span class="detail-label">"Channels:"</span>
                        <span class="detail-value">{channels}</span>
                    </div>
                </div>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Disks:"</span>
                        <span class="detail-value">{disks}</span>
                    </div>
                    <div class="detail-group">
                        <span class="detail-label">"Media Grading:"</span>
                        <span class="detail-value">{media}</span>
                    </div>
                    <div class="detail-group">
                        <span class="detail-label">"Sleeve Grading:"</span>
                        <span class="detail-value">{sleeve}</span>
                    </div>
                </div>
            </div>
        }
        .into_view()
    } else if let Some(ref cd) = details.cd_details {
        let disks = cd
            .disks
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());
        view! {
            <div class="detail-section">
                <h4>"CD Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Disks:"</span>
                        <span class="detail-value">{disks}</span>
                    </div>
                </div>
            </div>
        }
        .into_view()
    } else if let Some(ref cas) = details.cassette_details {
        let cassettes = cas
            .cassettes
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());
        view! {
            <div class="detail-section">
                <h4>"Cassette Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Cassettes:"</span>
                        <span class="detail-value">{cassettes}</span>
                    </div>
                </div>
            </div>
        }
        .into_view()
    } else if let Some(ref dvd) = details.dvd_details {
        let disks = dvd
            .disks
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());
        view! {
            <div class="detail-section">
                <h4>"DVD Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Disks:"</span>
                        <span class="detail-value">{disks}</span>
                    </div>
                </div>
            </div>
        }
        .into_view()
    } else {
        ().into_view()
    }
}

fn render_state_details(details: &ItemFullDetails) -> View {
    if let Some(ref loan) = details.loan_details {
        let date_loaned = loan.date_loaned.format("%Y-%m-%d").to_string();
        let date_due_back = loan
            .date_due_back
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "-".to_string());
        let loaned_to = loan.loaned_to.clone();
        view! {
            <div class="detail-section">
                <h4>"Loan Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Date Loaned:"</span>
                        <span class="detail-value">{date_loaned}</span>
                    </div>
                    <div class="detail-group">
                        <span class="detail-label">"Date Due Back:"</span>
                        <span class="detail-value">{date_due_back}</span>
                    </div>
                    <div class="detail-group">
                        <span class="detail-label">"Loaned To:"</span>
                        <span class="detail-value">{loaned_to}</span>
                    </div>
                </div>
            </div>
        }
        .into_view()
    } else if let Some(ref missing) = details.missing_details {
        let date_missing = missing.date_missing.format("%Y-%m-%d").to_string();
        view! {
            <div class="detail-section">
                <h4>"Missing Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Date Missing:"</span>
                        <span class="detail-value">{date_missing}</span>
                    </div>
                </div>
            </div>
        }
        .into_view()
    } else if let Some(ref disposed) = details.disposed_details {
        let date_disposed = disposed.date_disposed.format("%Y-%m-%d").to_string();
        view! {
            <div class="detail-section">
                <h4>"Disposed Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Date Disposed:"</span>
                        <span class="detail-value">{date_disposed}</span>
                    </div>
                </div>
            </div>
        }
        .into_view()
    } else {
        ().into_view()
    }
}

#[component]
fn ItemExpandedRow(
    item: Item,
    location_name: String,
    #[prop(default = String::new())] search_query: String,
    org_id: Uuid,
) -> impl IntoView {
    let item_id = item.id;
    let date_acquired = item
        .date_acquired
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string());

    let date_entered = item.date_entered.format("%Y-%m-%d %H:%M").to_string();

    let details_resource = create_resource(
        move || (org_id, item_id),
        move |(org_id, item_id)| async move { get_item_details(org_id, item_id).await },
    );

    view! {
        <tr class="item-expanded">
            <td colspan="4">
                <div class="item-details">
                    <div class="detail-row">
                        <div class="detail-group">
                            <span class="detail-label">"Description:"</span>
                            <span class="detail-value">
                                {highlight_match(&item.description.unwrap_or_else(|| "-".to_string()), &search_query)}
                            </span>
                        </div>
                    </div>
                    <div class="detail-row">
                        <div class="detail-group">
                            <span class="detail-label">"Notes:"</span>
                            <span class="detail-value">
                                {highlight_match(&item.notes.unwrap_or_else(|| "-".to_string()), &search_query)}
                            </span>
                        </div>
                    </div>
                    <div class="detail-row">
                        <div class="detail-group">
                            <span class="detail-label">"Location:"</span>
                            <span class="detail-value">{location_name}</span>
                        </div>
                        <div class="detail-group">
                            <span class="detail-label">"Date Acquired:"</span>
                            <span class="detail-value">{date_acquired}</span>
                        </div>
                        <div class="detail-group">
                            <span class="detail-label">"Date Entered:"</span>
                            <span class="detail-value">{date_entered}</span>
                        </div>
                    </div>
                    <Suspense fallback=move || view! { <div class="loading">"Loading details..."</div> }>
                        {move || {
                            details_resource.get().map(|result| match result {
                                Ok(details) => {
                                    let type_view = render_type_details(&details);
                                    let state_view = render_state_details(&details);
                                    view! {
                                        {type_view}
                                        {state_view}
                                    }.into_view()
                                }
                                Err(_) => ().into_view(),
                            })
                        }}
                    </Suspense>
                </div>
            </td>
        </tr>
    }
}
