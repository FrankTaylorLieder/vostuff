use leptos::*;
use std::collections::HashMap;
use uuid::Uuid;

use pulldown_cmark::{Options, Parser, html};

use crate::server_fns::items::{
    Item, ItemFullDetails, ItemState, Location, UpdateItemRequest, get_item_details, update_item,
};

fn render_markdown(text: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;
    let parser = Parser::new_ext(text, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

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
    #[prop(default = vec![])] locations_list: Vec<Location>,
    #[prop(default = String::new())] search_query: String,
    #[prop(default = "name".to_string())] sort_by: String,
    #[prop(default = "asc".to_string())] sort_order: String,
    #[prop(optional)] set_sort_by: Option<WriteSignal<String>>,
    #[prop(optional)] set_sort_order: Option<WriteSignal<String>>,
    #[prop(optional)] on_item_updated: Option<Callback<()>>,
    #[prop(optional)] expanded_row: Option<ReadSignal<Option<Uuid>>>,
    #[prop(optional)] set_expanded_row: Option<WriteSignal<Option<Uuid>>>,
    org_id: Uuid,
) -> impl IntoView {
    let locations_list = store_value(locations_list);
    let (local_expanded, local_set_expanded) = create_signal::<Option<Uuid>>(None);
    let expanded_row = expanded_row.unwrap_or(local_expanded);
    let set_expanded_row = set_expanded_row.unwrap_or(local_set_expanded);

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

    let on_type = make_sort_handler("kind");
    let on_name = make_sort_handler("name");
    let on_state = make_sort_handler("state");
    let on_location = make_sort_handler("location_id");

    let ind_type = sort_indicator("kind");
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
                                <td class="col-type">{item.kind_name.clone()}</td>
                                <td class="col-name">{highlight_match(&item.name, &sq)}</td>
                                <td class="col-state">
                                    <span class=format!("state-badge {}", item.state.css_class())>
                                        {item.state.display_name()}
                                    </span>
                                </td>
                                <td class="col-location">{location_name.clone()}</td>
                            </tr>
                            <Show when=is_expanded fallback=|| ()>
                                <ItemExpandedRow
                                    item=item_for_details.clone()
                                    location_name=location_name.clone()
                                    search_query=sq2.clone()
                                    org_id=org_id
                                    locations_list=locations_list.get_value()
                                    on_item_updated=on_item_updated.unwrap_or(Callback::new(|_| {}))
                                />
                            </Show>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

fn value_to_edit_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        _ => v.to_string(),
    }
}

fn format_field_name(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_soft_fields(soft_fields: &serde_json::Value) -> View {
    let Some(obj) = soft_fields.as_object() else {
        return ().into_view();
    };
    if obj.is_empty() {
        return ().into_view();
    }
    let fields: Vec<(String, String)> = obj
        .iter()
        .map(|(k, v)| (format_field_name(k), value_to_edit_str(v)))
        .collect();
    view! {
        <div class="detail-section">
            <h4>"Details"</h4>
            <div class="detail-row">
                {fields
                    .into_iter()
                    .map(|(key, value)| {
                        view! {
                            <div class="detail-group">
                                <span class="detail-label">{key + ":"}</span>
                                <span class="detail-value">{value}</span>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
    .into_view()
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
    #[prop(default = vec![])] locations_list: Vec<Location>,
    on_item_updated: Callback<()>,
) -> impl IntoView {
    let item_id = item.id;
    let date_acquired = item
        .date_acquired
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string());

    let date_entered = item.date_entered.format("%Y-%m-%d %H:%M").to_string();

    let (editing, set_editing) = create_signal(false);
    let (saving, set_saving) = create_signal(false);

    // Editable signals for base fields
    let (edit_name, set_edit_name) = create_signal(item.name.clone());
    let (edit_description, set_edit_description) =
        create_signal(item.description.clone().unwrap_or_default());
    let (edit_notes, set_edit_notes) = create_signal(item.notes.clone().unwrap_or_default());
    let (edit_location_id, set_edit_location_id) = create_signal(
        item.location_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
    );
    let (edit_date_acquired, set_edit_date_acquired) = create_signal(
        item.date_acquired
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
    );

    // Soft field signals — one per entry in item.soft_fields
    let soft_field_entries: Vec<(String, ReadSignal<String>, WriteSignal<String>)> = item
        .soft_fields
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| {
            let (r, w) = create_signal(value_to_edit_str(&v));
            (k, r, w)
        })
        .collect();
    let soft_field_entries = store_value(soft_field_entries);
    let orig_soft_fields = store_value(item.soft_fields.clone());

    // Loan signals
    let (edit_loan_date_loaned, set_edit_loan_date_loaned) = create_signal(String::new());
    let (edit_loan_date_due_back, set_edit_loan_date_due_back) = create_signal(String::new());
    let (edit_loan_loaned_to, set_edit_loan_loaned_to) = create_signal(String::new());

    // Missing/Disposed signals
    let (edit_missing_date, set_edit_missing_date) = create_signal(String::new());
    let (edit_disposed_date, set_edit_disposed_date) = create_signal(String::new());

    // Store fetched details for initializing edit signals
    let (fetched_details, set_fetched_details) = create_signal::<Option<ItemFullDetails>>(None);

    let (details_version, set_details_version) = create_signal(0u32);

    let details_resource = create_resource(
        move || (org_id, item_id, details_version.get()),
        move |(org_id, item_id, _)| async move { get_item_details(org_id, item_id).await },
    );

    // Initialize edit signals from details when entering edit mode
    let init_edit_from_details = move || {
        if let Some(details) = fetched_details.get() {
            if let Some(obj) = details.item.soft_fields.as_object() {
                for (k, _, set_w) in soft_field_entries.get_value().iter() {
                    let val = obj.get(k).map(value_to_edit_str).unwrap_or_default();
                    set_w.set(val);
                }
            }
            if let Some(ref loan) = details.loan_details {
                set_edit_loan_date_loaned.set(loan.date_loaned.format("%Y-%m-%d").to_string());
                set_edit_loan_date_due_back.set(
                    loan.date_due_back
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default(),
                );
                set_edit_loan_loaned_to.set(loan.loaned_to.clone());
            }
            if let Some(ref missing) = details.missing_details {
                set_edit_missing_date.set(missing.date_missing.format("%Y-%m-%d").to_string());
            }
            if let Some(ref disposed) = details.disposed_details {
                set_edit_disposed_date.set(disposed.date_disposed.format("%Y-%m-%d").to_string());
            }
        }
    };

    let orig_name = store_value(item.name.clone());
    let orig_description = store_value(item.description.clone().unwrap_or_default());
    let orig_notes = store_value(item.notes.clone().unwrap_or_default());
    let orig_location_id = store_value(
        item.location_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
    );
    let orig_date_acquired = store_value(
        item.date_acquired
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
    );

    let cancel_edit = move || {
        set_edit_name.set(orig_name.get_value());
        set_edit_description.set(orig_description.get_value());
        set_edit_notes.set(orig_notes.get_value());
        set_edit_location_id.set(orig_location_id.get_value());
        set_edit_date_acquired.set(orig_date_acquired.get_value());
        if let Some(obj) = orig_soft_fields.get_value().as_object() {
            for (k, _, set_w) in soft_field_entries.get_value().iter() {
                let val = obj.get(k).map(value_to_edit_str).unwrap_or_default();
                set_w.set(val);
            }
        }
        init_edit_from_details();
        set_editing.set(false);
    };

    let item_state_for_save = store_value(item.state.clone());

    let save_action = create_action(move |_: &()| {
        let is = item_state_for_save.get_value();
        let name = edit_name.get();
        let description = edit_description.get();
        let notes = edit_notes.get();
        let location_str = edit_location_id.get();
        let date_acq_str = edit_date_acquired.get();

        // Collect soft fields
        let sf_map: serde_json::Map<String, serde_json::Value> = soft_field_entries
            .get_value()
            .iter()
            .map(|(k, r, _)| {
                let s = r.get();
                let v = if s.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(s)
                };
                (k.clone(), v)
            })
            .collect();

        let mut req = UpdateItemRequest {
            name: Some(name),
            description: Some(description),
            notes: Some(notes),
            location_id: if location_str.is_empty() {
                None
            } else {
                Uuid::parse_str(&location_str).ok()
            },
            date_acquired: if date_acq_str.is_empty() {
                None
            } else {
                chrono::NaiveDate::parse_from_str(&date_acq_str, "%Y-%m-%d").ok()
            },
            state: None,
            soft_fields: Some(serde_json::Value::Object(sf_map)),
            loan_date_loaned: None,
            loan_date_due_back: None,
            loan_loaned_to: None,
            missing_date_missing: None,
            disposed_date_disposed: None,
        };

        // State-specific fields
        match is {
            ItemState::Loaned => {
                let dl = edit_loan_date_loaned.get();
                if !dl.is_empty() {
                    req.loan_date_loaned = chrono::NaiveDate::parse_from_str(&dl, "%Y-%m-%d").ok();
                }
                let ddb = edit_loan_date_due_back.get();
                if !ddb.is_empty() {
                    req.loan_date_due_back =
                        chrono::NaiveDate::parse_from_str(&ddb, "%Y-%m-%d").ok();
                }
                let lt = edit_loan_loaned_to.get();
                if !lt.is_empty() {
                    req.loan_loaned_to = Some(lt);
                }
            }
            ItemState::Missing => {
                let dm = edit_missing_date.get();
                if !dm.is_empty() {
                    req.missing_date_missing =
                        chrono::NaiveDate::parse_from_str(&dm, "%Y-%m-%d").ok();
                }
            }
            ItemState::Disposed => {
                let dd = edit_disposed_date.get();
                if !dd.is_empty() {
                    req.disposed_date_disposed =
                        chrono::NaiveDate::parse_from_str(&dd, "%Y-%m-%d").ok();
                }
            }
            _ => {}
        }

        async move { update_item(org_id, item_id, req).await }
    });

    // React to save action completion
    create_effect(move |_| {
        if let Some(result) = save_action.value().get() {
            match result {
                Ok(()) => {
                    set_saving.set(false);
                    set_editing.set(false);
                    set_details_version.update(|v| *v += 1);
                    on_item_updated.call(());
                }
                Err(e) => {
                    set_saving.set(false);
                    leptos::logging::error!("Failed to save item: {}", e);
                }
            }
        }
    });

    let locations_for_edit = locations_list.clone();
    let item_state_for_view = item.state.clone();
    let kind_name_for_edit = item.kind_name.clone();

    view! {
        <tr class="item-expanded" on:click=|e| e.stop_propagation()>
            <td colspan="4">
                <div class="item-details">
                    <Show
                        when=move || editing.get()
                        fallback={
                            let location_name = location_name.clone();
                            let date_acquired = date_acquired.clone();
                            let date_entered = date_entered.clone();
                            let search_query = search_query.clone();
                            let item = item.clone();
                            move || {
                                let description_text = item.description.clone().unwrap_or_else(|| "-".to_string());
                                let notes_text = item.notes.clone().unwrap_or_else(|| "-".to_string());
                                let soft_fields_view = render_soft_fields(&item.soft_fields);
                                view! {
                                    <div class="detail-row">
                                        <div class="detail-group">
                                            <span class="detail-label">"Description:"</span>
                                            <span class="detail-value">
                                                {highlight_match(&description_text, &search_query)}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="detail-row">
                                        <div class="detail-group">
                                            <span class="detail-label">"Notes:"</span>
                                            <div class="detail-value markdown-content" inner_html=render_markdown(&notes_text)></div>
                                        </div>
                                    </div>
                                    <div class="detail-row">
                                        <div class="detail-group">
                                            <span class="detail-label">"Location:"</span>
                                            <span class="detail-value">{location_name.clone()}</span>
                                        </div>
                                        <div class="detail-group">
                                            <span class="detail-label">"Date Acquired:"</span>
                                            <span class="detail-value">{date_acquired.clone()}</span>
                                        </div>
                                        <div class="detail-group">
                                            <span class="detail-label">"Date Entered:"</span>
                                            <span class="detail-value">{date_entered.clone()}</span>
                                        </div>
                                    </div>
                                    {soft_fields_view}
                                    <Suspense fallback=move || view! { <div class="loading">"Loading details..."</div> }>
                                        {move || {
                                            details_resource.get().map(|result| match result {
                                                Ok(details) => {
                                                    set_fetched_details.set(Some(details.clone()));
                                                    let state_view = render_state_details(&details);
                                                    view! {
                                                        {state_view}
                                                    }.into_view()
                                                }
                                                Err(_) => ().into_view(),
                                            })
                                        }}
                                    </Suspense>
                                    <div class="detail-actions">
                                        <button
                                            class="btn btn-edit"
                                            on:click=move |_| {
                                                init_edit_from_details();
                                                set_editing.set(true);
                                            }
                                        >
                                            "Edit"
                                        </button>
                                    </div>
                                }.into_view()
                            }
                        }
                    >
                        {
                            let locations_for_edit = locations_for_edit.clone();
                            let item_state_for_view = item_state_for_view.clone();
                            let kind_name_for_edit = kind_name_for_edit.clone();
                            move || {
                                let locs = locations_for_edit.clone();
                                let is = item_state_for_view.clone();
                                view! {
                                    <div class="detail-row">
                                        <div class="detail-group">
                                            <span class="detail-label">"Name:"</span>
                                            <input
                                                type="text"
                                                class="edit-input"
                                                prop:value=edit_name
                                                on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="detail-row">
                                        <div class="detail-group">
                                            <span class="detail-label">"Description:"</span>
                                            <input
                                                type="text"
                                                class="edit-input"
                                                prop:value=edit_description
                                                on:input=move |ev| set_edit_description.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="detail-row">
                                        <div class="detail-group">
                                            <span class="detail-label">"Notes:"</span>
                                            <textarea
                                                class="edit-textarea"
                                                prop:value=edit_notes
                                                on:input=move |ev| set_edit_notes.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="detail-row">
                                        <div class="detail-group">
                                            <span class="detail-label">"Location:"</span>
                                            <select
                                                class="edit-select"
                                                prop:value=edit_location_id
                                                on:change=move |ev| set_edit_location_id.set(event_target_value(&ev))
                                            >
                                                <option value="">"- None -"</option>
                                                {locs
                                                    .iter()
                                                    .map(|loc| {
                                                        let val = loc.id.to_string();
                                                        let name = loc.name.clone();
                                                        view! { <option value=val>{name}</option> }
                                                    })
                                                    .collect_view()}
                                            </select>
                                        </div>
                                        <div class="detail-group">
                                            <span class="detail-label">"Date Acquired:"</span>
                                            <input
                                                type="date"
                                                class="edit-input"
                                                prop:value=edit_date_acquired
                                                on:input=move |ev| set_edit_date_acquired.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="detail-group">
                                            <span class="detail-label">"Type:"</span>
                                            <span class="detail-value">{kind_name_for_edit.clone()}</span>
                                        </div>
                                    </div>

                                    // Soft fields edit section
                                    {
                                        let entries = soft_field_entries.get_value();
                                        if entries.is_empty() {
                                            ().into_view()
                                        } else {
                                            view! {
                                                <div class="detail-section">
                                                    <h4>"Details"</h4>
                                                    {entries
                                                        .into_iter()
                                                        .map(|(k, r, set_w)| {
                                                            let label = format_field_name(&k);
                                                            view! {
                                                                <div class="detail-row">
                                                                    <div class="detail-group">
                                                                        <span class="detail-label">{label + ":"}</span>
                                                                        <input
                                                                            type="text"
                                                                            class="edit-input"
                                                                            prop:value=r
                                                                            on:input=move |ev| set_w.set(event_target_value(&ev))
                                                                        />
                                                                    </div>
                                                                </div>
                                                            }
                                                        })
                                                        .collect_view()}
                                                </div>
                                            }
                                            .into_view()
                                        }
                                    }

                                    // State-specific edit fields
                                    {render_state_edit_fields(&is, edit_loan_date_loaned, set_edit_loan_date_loaned, edit_loan_date_due_back, set_edit_loan_date_due_back, edit_loan_loaned_to, set_edit_loan_loaned_to, edit_missing_date, set_edit_missing_date, edit_disposed_date, set_edit_disposed_date)}

                                    <div class="detail-actions">
                                        <button
                                            class="btn btn-ok"
                                            prop:disabled=saving
                                            on:click=move |_| {
                                                set_saving.set(true);
                                                save_action.dispatch(());
                                            }
                                        >
                                            {move || if saving.get() { "Saving..." } else { "OK" }}
                                        </button>
                                        <button
                                            class="btn btn-cancel"
                                            prop:disabled=saving
                                            on:click=move |_| cancel_edit()
                                        >
                                            "Cancel"
                                        </button>
                                    </div>
                                }.into_view()
                            }
                        }
                    </Show>
                </div>
            </td>
        </tr>
    }
}

#[allow(clippy::too_many_arguments)]
fn render_state_edit_fields(
    state: &ItemState,
    edit_loan_date_loaned: ReadSignal<String>,
    set_edit_loan_date_loaned: WriteSignal<String>,
    edit_loan_date_due_back: ReadSignal<String>,
    set_edit_loan_date_due_back: WriteSignal<String>,
    edit_loan_loaned_to: ReadSignal<String>,
    set_edit_loan_loaned_to: WriteSignal<String>,
    edit_missing_date: ReadSignal<String>,
    set_edit_missing_date: WriteSignal<String>,
    edit_disposed_date: ReadSignal<String>,
    set_edit_disposed_date: WriteSignal<String>,
) -> View {
    match state {
        ItemState::Loaned => view! {
            <div class="detail-section">
                <h4>"Loan Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Date Loaned:"</span>
                        <input type="date" class="edit-input" prop:value=edit_loan_date_loaned on:input=move |ev| set_edit_loan_date_loaned.set(event_target_value(&ev)) />
                    </div>
                    <div class="detail-group">
                        <span class="detail-label">"Date Due Back:"</span>
                        <input type="date" class="edit-input" prop:value=edit_loan_date_due_back on:input=move |ev| set_edit_loan_date_due_back.set(event_target_value(&ev)) />
                    </div>
                    <div class="detail-group">
                        <span class="detail-label">"Loaned To:"</span>
                        <input type="text" class="edit-input" prop:value=edit_loan_loaned_to on:input=move |ev| set_edit_loan_loaned_to.set(event_target_value(&ev)) />
                    </div>
                </div>
            </div>
        }.into_view(),
        ItemState::Missing => view! {
            <div class="detail-section">
                <h4>"Missing Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Date Missing:"</span>
                        <input type="date" class="edit-input" prop:value=edit_missing_date on:input=move |ev| set_edit_missing_date.set(event_target_value(&ev)) />
                    </div>
                </div>
            </div>
        }.into_view(),
        ItemState::Disposed => view! {
            <div class="detail-section">
                <h4>"Disposed Details"</h4>
                <div class="detail-row">
                    <div class="detail-group">
                        <span class="detail-label">"Date Disposed:"</span>
                        <input type="date" class="edit-input" prop:value=edit_disposed_date on:input=move |ev| set_edit_disposed_date.set(event_target_value(&ev)) />
                    </div>
                </div>
            </div>
        }.into_view(),
        _ => ().into_view(),
    }
}
