use leptos::*;
use std::collections::HashMap;
use uuid::Uuid;

use pulldown_cmark::{Options, Parser, html};

use crate::components::soft_field_helpers::{
    format_field_name, format_soft_field_value, render_soft_field_input, value_to_edit_str,
};
use crate::server_fns::items::{
    Item, ItemFullDetails, ItemState, Location, UpdateItemRequest, get_item_details, update_item,
};
use crate::server_fns::kinds::{get_kind_fields, KindFieldDef};

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

fn render_soft_fields_with_defs(
    soft_fields: &serde_json::Value,
    kind_fields: &[KindFieldDef],
) -> View {
    let Some(obj) = soft_fields.as_object() else {
        return ().into_view();
    };

    let mut field_names_in_kind: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut pairs: Vec<(String, String)> = Vec::new();

    let mut sorted_fields: Vec<&KindFieldDef> = kind_fields.iter().collect();
    sorted_fields.sort_by_key(|f| f.display_order);

    for field_def in &sorted_fields {
        field_names_in_kind.insert(field_def.name.clone());
        let Some(raw_val) = obj.get(&field_def.name) else {
            continue;
        };
        if raw_val.is_null() {
            continue;
        }
        let raw_str = value_to_edit_str(raw_val);
        if raw_str.is_empty() {
            continue;
        }
        let label = field_def
            .display_name
            .as_deref()
            .unwrap_or(&field_def.name)
            .to_string();
        let value = format_soft_field_value(&field_def.field_type, raw_val, &field_def.enum_values);
        pairs.push((label, value));
    }

    // Orphaned fields: in soft_fields but not in the kind definition
    for (k, v) in obj.iter() {
        if field_names_in_kind.contains(k) {
            continue;
        }
        if v.is_null() {
            continue;
        }
        let s = value_to_edit_str(v);
        if s.is_empty() {
            continue;
        }
        pairs.push((format_field_name(k), s));
    }

    if pairs.is_empty() {
        return ().into_view();
    }

    view! {
        <div class="detail-section">
            <h4>"Details"</h4>
            <div class="detail-row">
                {pairs
                    .into_iter()
                    .map(|(label, value)| view! {
                        <div class="detail-group">
                            <span class="detail-label">{label + ":"}</span>
                            <span class="detail-value">{value}</span>
                        </div>
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

    // Soft field signals — store serde_json::Value directly so types are
    // preserved through edit and save without any guessing at save time.
    let init_map: HashMap<String, serde_json::Value> = item
        .soft_fields
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let orig_soft_field_map = store_value(init_map.clone());
    let soft_field_map = create_rw_signal(init_map);
    let soft_fields_stored = store_value(item.soft_fields.clone());
    let kind_id_stored = item.kind_id;

    // Use spawn_local (not create_resource) to avoid triggering the parent <Suspense>
    // boundary in home.rs, which would cause a scroll-to-top on row expand.
    let kind_fields = create_rw_signal::<Vec<KindFieldDef>>(vec![]);
    spawn_local(async move {
        if let Ok(fields) = get_kind_fields(org_id, kind_id_stored).await {
            kind_fields.set(fields);
        }
    });

    let (save_error, set_save_error) = create_signal::<Option<String>>(None);

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
            // Update base fields from the freshly-fetched details so re-entering
            // edit mode after a save shows the current saved values.
            set_edit_name.set(details.item.name.clone());
            set_edit_description
                .set(details.item.description.clone().unwrap_or_default());
            set_edit_notes.set(details.item.notes.clone().unwrap_or_default());
            set_edit_location_id.set(
                details
                    .item
                    .location_id
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
            );
            set_edit_date_acquired.set(
                details
                    .item
                    .date_acquired
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
            );
            if let Some(obj) = details.item.soft_fields.as_object() {
                soft_field_map.update(|m| {
                    for (k, v) in obj.iter() {
                        m.insert(k.clone(), v.clone());
                    }
                });
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
        soft_field_map.set(orig_soft_field_map.get_value());
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

        // Values are already correctly typed (stored as serde_json::Value by
        // the input handlers), so no conversion is needed here.
        let sf_map: serde_json::Map<String, serde_json::Value> =
            soft_field_map.get_untracked().into_iter().collect();

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
            // Serialize to a JSON string — serde_urlencoded (used by Leptos
            // server fn transport) loses type info for nested serde_json::Value,
            // so we pass it as a plain string and parse it back server-side.
            soft_fields: serde_json::to_string(&serde_json::Value::Object(sf_map)).ok(),
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
                    let msg = format!("{}", e);
                    leptos::logging::error!("Failed to save item: {}", msg);
                    set_save_error.set(Some(msg));
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
                                    {move || {
                                        let sf = soft_fields_stored.get_value();
                                        let fields = kind_fields.get();
                                        if fields.is_empty() {
                                            render_soft_fields(&sf)
                                        } else {
                                            render_soft_fields_with_defs(&sf, &fields)
                                        }
                                    }}
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
                                    <div class="form-group">
                                        <label class="form-label">"Name"</label>
                                        <input
                                            type="text"
                                            class="form-control"
                                            prop:value=edit_name
                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label class="form-label">"Description"</label>
                                        <input
                                            type="text"
                                            class="form-control"
                                            prop:value=edit_description
                                            on:input=move |ev| set_edit_description.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label class="form-label">"Notes"</label>
                                        <textarea
                                            class="form-control"
                                            style="min-height:80px;resize:vertical;"
                                            prop:value=edit_notes
                                            on:input=move |ev| set_edit_notes.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label class="form-label">"Location"</label>
                                        <select
                                            class="form-control"
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
                                    <div class="form-group">
                                        <label class="form-label">"Date Acquired"</label>
                                        <input
                                            type="date"
                                            class="form-control"
                                            prop:value=edit_date_acquired
                                            on:input=move |ev| set_edit_date_acquired.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label class="form-label">"Type"</label>
                                        <span class="detail-value">{kind_name_for_edit.clone()}</span>
                                    </div>

                                    // Soft fields edit section — type-specific inputs
                                    {move || {
                                        let fields = kind_fields.get();
                                        if fields.is_empty() {
                                            render_soft_fields_edit_fallback(soft_field_map)
                                        } else {
                                            render_soft_fields_edit_with_defs(&fields, soft_field_map)
                                        }
                                    }}

                                    // State-specific edit fields
                                    {render_state_edit_fields(&is, edit_loan_date_loaned, set_edit_loan_date_loaned, edit_loan_date_due_back, set_edit_loan_date_due_back, edit_loan_loaned_to, set_edit_loan_loaned_to, edit_missing_date, set_edit_missing_date, edit_disposed_date, set_edit_disposed_date)}

                                    <Show when=move || save_error.get().is_some() fallback=|| ()>
                                        <div class="error">
                                            {move || save_error.get().unwrap_or_default()}
                                        </div>
                                    </Show>
                                    <div class="detail-actions">
                                        <button
                                            class="btn btn-secondary"
                                            prop:disabled=saving
                                            on:click=move |_| cancel_edit()
                                        >
                                            "Cancel"
                                        </button>
                                        <button
                                            class="btn btn-primary"
                                            style="width:auto;"
                                            prop:disabled=saving
                                            on:click=move |_| {
                                                set_save_error.set(None);
                                                set_saving.set(true);
                                                save_action.dispatch(());
                                            }
                                        >
                                            {move || if saving.get() { "Saving..." } else { "Save" }}
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
            <div>
                <div style="font-size:12px;font-weight:600;color:#777;text-transform:uppercase;letter-spacing:0.5px;margin:16px 0 8px;">
                    "Loan Details"
                </div>
                <div class="form-group">
                    <label class="form-label">"Date Loaned"</label>
                    <input type="date" class="form-control" prop:value=edit_loan_date_loaned on:input=move |ev| set_edit_loan_date_loaned.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label class="form-label">"Date Due Back"</label>
                    <input type="date" class="form-control" prop:value=edit_loan_date_due_back on:input=move |ev| set_edit_loan_date_due_back.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label class="form-label">"Loaned To"</label>
                    <input type="text" class="form-control" prop:value=edit_loan_loaned_to on:input=move |ev| set_edit_loan_loaned_to.set(event_target_value(&ev)) />
                </div>
            </div>
        }.into_view(),
        ItemState::Missing => view! {
            <div>
                <div style="font-size:12px;font-weight:600;color:#777;text-transform:uppercase;letter-spacing:0.5px;margin:16px 0 8px;">
                    "Missing Details"
                </div>
                <div class="form-group">
                    <label class="form-label">"Date Missing"</label>
                    <input type="date" class="form-control" prop:value=edit_missing_date on:input=move |ev| set_edit_missing_date.set(event_target_value(&ev)) />
                </div>
            </div>
        }.into_view(),
        ItemState::Disposed => view! {
            <div>
                <div style="font-size:12px;font-weight:600;color:#777;text-transform:uppercase;letter-spacing:0.5px;margin:16px 0 8px;">
                    "Disposed Details"
                </div>
                <div class="form-group">
                    <label class="form-label">"Date Disposed"</label>
                    <input type="date" class="form-control" prop:value=edit_disposed_date on:input=move |ev| set_edit_disposed_date.set(event_target_value(&ev)) />
                </div>
            </div>
        }.into_view(),
        _ => ().into_view(),
    }
}

fn render_soft_fields_edit_with_defs(
    kind_fields: &[KindFieldDef],
    soft_field_map: RwSignal<HashMap<String, serde_json::Value>>,
) -> View {
    if kind_fields.is_empty() {
        return ().into_view();
    }

    let mut sorted: Vec<KindFieldDef> = kind_fields.to_vec();
    sorted.sort_by_key(|f| f.display_order);

    view! {
        <div>
            <div style="font-size:12px;font-weight:600;color:#777;text-transform:uppercase;letter-spacing:0.5px;margin:16px 0 8px;">
                "Details"
            </div>
            {sorted
                .into_iter()
                .map(|field_def| {
                    let name = field_def.name.clone();
                    let label = field_def
                        .display_name
                        .clone()
                        .unwrap_or_else(|| format_field_name(&name));
                    let ft = field_def.field_type.clone();
                    let enum_values = field_def.enum_values.clone();
                    view! {
                        <div class="form-group">
                            <label class="form-label">{label}</label>
                            {render_soft_field_input(name, ft, enum_values, soft_field_map)}
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_view()
}

fn render_soft_fields_edit_fallback(
    soft_field_map: RwSignal<HashMap<String, serde_json::Value>>,
) -> View {
    let entries: Vec<String> = soft_field_map.get_untracked().into_keys().collect();
    if entries.is_empty() {
        return ().into_view();
    }
    view! {
        <div class="detail-section">
            <h4>"Details"</h4>
            {entries
                .into_iter()
                .map(|k| {
                    let label = format_field_name(&k);
                    let name = k.clone();
                    view! {
                        <div class="detail-row">
                            <div class="detail-group">
                                <span class="detail-label">{label + ":"}</span>
                                <input
                                    type="text"
                                    class="edit-input"
                                    prop:value=move || {
                                        soft_field_map.with(|m| {
                                            m.get(&name).map(|v| value_to_edit_str(v)).unwrap_or_default()
                                        })
                                    }
                                    on:input=move |ev| {
                                        let s = event_target_value(&ev);
                                        soft_field_map.update(|m| {
                                            // Preserve the original JSON type when re-entering a value
                                            let v = match m.get(&k) {
                                                Some(serde_json::Value::Number(_)) => {
                                                    if let Ok(i) = s.parse::<i64>() { serde_json::json!(i) }
                                                    else if let Ok(f) = s.parse::<f64>() { serde_json::json!(f) }
                                                    else { serde_json::Value::String(s.clone()) }
                                                }
                                                Some(serde_json::Value::Bool(_)) => {
                                                    serde_json::Value::Bool(s == "true")
                                                }
                                                _ => serde_json::Value::String(s.clone()),
                                            };
                                            m.insert(k.clone(), v);
                                        });
                                    }
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
