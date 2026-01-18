use leptos::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::server_fns::items::Item;

#[component]
pub fn ItemsTable(items: Vec<Item>, locations: HashMap<Uuid, String>) -> impl IntoView {
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

    view! {
        <table class="items-table">
            <thead>
                <tr>
                    <th class="col-type">"Type"</th>
                    <th class="col-name">"Name"</th>
                    <th class="col-state">"State"</th>
                    <th class="col-location">"Location"</th>
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
                        view! {
                            <tr
                                class="item-row"
                                class:expanded=is_expanded
                                on:click=move |_| toggle_row(item_id)
                            >
                                <td class="col-type">{item.item_type.display_name()}</td>
                                <td class="col-name">{item.name.clone()}</td>
                                <td class="col-state">
                                    <span class=format!("state-badge {}", item.state.css_class())>
                                        {item.state.display_name()}
                                    </span>
                                </td>
                                <td class="col-location">{location_name.clone()}</td>
                            </tr>
                            <Show when=is_expanded fallback=|| ()>
                                <ItemExpandedRow item=item_for_details.clone() location_name=location_name.clone()/>
                            </Show>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

#[component]
fn ItemExpandedRow(item: Item, location_name: String) -> impl IntoView {
    let date_acquired = item
        .date_acquired
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string());

    let date_entered = item.date_entered.format("%Y-%m-%d %H:%M").to_string();

    view! {
        <tr class="item-expanded">
            <td colspan="4">
                <div class="item-details">
                    <div class="detail-row">
                        <div class="detail-group">
                            <span class="detail-label">"Description:"</span>
                            <span class="detail-value">
                                {item.description.unwrap_or_else(|| "-".to_string())}
                            </span>
                        </div>
                    </div>
                    <div class="detail-row">
                        <div class="detail-group">
                            <span class="detail-label">"Notes:"</span>
                            <span class="detail-value">
                                {item.notes.unwrap_or_else(|| "-".to_string())}
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
                </div>
            </td>
        </tr>
    }
}
