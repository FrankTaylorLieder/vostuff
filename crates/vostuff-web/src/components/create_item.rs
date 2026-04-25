use leptos::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::components::soft_field_helpers::{format_field_name, render_soft_field_input};
use crate::server_fns::items::{CreateItemRequest, Location, create_item, get_locations};
use crate::server_fns::kinds::{KindFieldDef, get_kind_fields, get_kinds};

#[component]
pub fn CreateItemModal(
    org_id: Uuid,
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let kind_id = create_rw_signal::<Option<Uuid>>(None);
    let name = create_rw_signal(String::new());
    let description = create_rw_signal(String::new());
    let notes = create_rw_signal(String::new());
    let location_id = create_rw_signal(String::new());
    let date_acquired = create_rw_signal(String::new());
    let soft_field_map = create_rw_signal::<HashMap<String, serde_json::Value>>(HashMap::new());
    let saving = create_rw_signal(false);
    let error = create_rw_signal::<Option<String>>(None);

    let reset_form = move || {
        kind_id.set(None);
        name.set(String::new());
        description.set(String::new());
        notes.set(String::new());
        location_id.set(String::new());
        date_acquired.set(String::new());
        soft_field_map.set(HashMap::new());
        saving.set(false);
        error.set(None);
    };

    // Clear soft fields when kind changes
    create_effect(move |prev: Option<Option<Uuid>>| {
        let current = kind_id.get();
        if prev.is_some() {
            soft_field_map.set(HashMap::new());
        }
        current
    });

    let kinds_resource = create_resource(
        move || org_id,
        |org_id| async move { get_kinds(org_id).await },
    );

    let locations_resource = create_resource(
        move || org_id,
        |org_id| async move { get_locations(org_id).await },
    );

    // Use spawn_local (not create_resource) to avoid triggering the parent Suspense boundary
    let kind_fields = create_rw_signal::<Vec<KindFieldDef>>(vec![]);
    create_effect(move |_| {
        let kid = kind_id.get();
        kind_fields.set(vec![]);
        if let Some(kid) = kid {
            spawn_local(async move {
                if let Ok(fields) = get_kind_fields(org_id, kid).await {
                    kind_fields.set(fields);
                }
            });
        }
    });

    let save_action = create_action(move |_: &()| {
        let kid = kind_id.get_untracked();
        let n = name.get_untracked();
        let desc = description.get_untracked();
        let nts = notes.get_untracked();
        let loc_str = location_id.get_untracked();
        let date_str = date_acquired.get_untracked();
        let raw_map = soft_field_map.get_untracked();

        async move {
            let kind_uuid = kid.ok_or_else(|| {
                leptos::server_fn::error::ServerFnError::<leptos::server_fn::error::NoCustomError>::ServerError(
                    "Please select a kind".to_string(),
                )
            })?;

            // Values are already correctly typed by the input handlers
            let sf_map: serde_json::Map<String, serde_json::Value> =
                raw_map.into_iter().collect();

            let req = CreateItemRequest {
                kind_id: kind_uuid,
                name: n,
                description: if desc.is_empty() { None } else { Some(desc) },
                notes: if nts.is_empty() { None } else { Some(nts) },
                location_id: if loc_str.is_empty() {
                    None
                } else {
                    Uuid::parse_str(&loc_str).ok()
                },
                date_acquired: if date_str.is_empty() {
                    None
                } else {
                    chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()
                },
                soft_fields: if sf_map.is_empty() {
                    None
                } else {
                    serde_json::to_string(&serde_json::Value::Object(sf_map)).ok()
                },
            };

            create_item(org_id, req).await
        }
    });

    create_effect(move |_| {
        if let Some(result) = save_action.value().get() {
            saving.set(false);
            match result {
                Ok(()) => {
                    on_created.call(());
                    on_close.call(());
                    reset_form();
                }
                Err(e) => {
                    error.set(Some(format!("{}", e)));
                }
            }
        }
    });

    let close_and_reset = move || {
        reset_form();
        on_close.call(());
    };

    view! {
        <Show when=move || show.get() fallback=|| ()>
            <div class="modal-overlay" on:click=move |_| close_and_reset()>
                <div class="modal" on:click=move |ev| ev.stop_propagation()>
                    <div class="modal-header">
                        <h2>"Add Item"</h2>
                    </div>
                    <div class="modal-body">
                        <div class="form-group">
                            <label>"Type"</label>
                            <Suspense fallback=|| view! { <span>"Loading..."</span> }>
                                {move || {
                                    let kinds = kinds_resource.get()
                                        .and_then(|r| r.ok())
                                        .unwrap_or_default();
                                    view! {
                                        <select
                                            class="form-control"
                                            prop:value=move || kind_id.get().map(|id| id.to_string()).unwrap_or_default()
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                kind_id.set(Uuid::parse_str(&val).ok());
                                            }
                                        >
                                            <option value="">"- Select Type -"</option>
                                            {kinds.into_iter().map(|k| {
                                                let val = k.id.to_string();
                                                let label = k.display_name.unwrap_or_else(|| k.name.clone());
                                                view! { <option value=val>{label}</option> }
                                            }).collect_view()}
                                        </select>
                                    }
                                }}
                            </Suspense>
                        </div>
                        <div class="form-group">
                            <label>"Name"</label>
                            <input
                                type="text"
                                class="form-control"
                                prop:value=name
                                on:input=move |ev| name.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-group">
                            <label>"Description"</label>
                            <input
                                type="text"
                                class="form-control"
                                prop:value=description
                                on:input=move |ev| description.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-group">
                            <label>"Notes"</label>
                            <textarea
                                class="form-control"
                                style="min-height:80px;resize:vertical;"
                                prop:value=notes
                                on:input=move |ev| notes.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-group">
                            <label>"Location"</label>
                            <Suspense fallback=|| view! { <span>"Loading..."</span> }>
                                {move || {
                                    let locs: Vec<Location> = locations_resource.get()
                                        .and_then(|r| r.ok())
                                        .unwrap_or_default();
                                    view! {
                                        <select
                                            class="form-control"
                                            prop:value=location_id
                                            on:change=move |ev| location_id.set(event_target_value(&ev))
                                        >
                                            <option value="">"- None -"</option>
                                            {locs.into_iter().map(|loc| {
                                                let val = loc.id.to_string();
                                                let lname = loc.name.clone();
                                                view! { <option value=val>{lname}</option> }
                                            }).collect_view()}
                                        </select>
                                    }
                                }}
                            </Suspense>
                        </div>
                        <div class="form-group">
                            <label>"Date Acquired"</label>
                            <input
                                type="date"
                                class="form-control"
                                prop:value=date_acquired
                                on:input=move |ev| date_acquired.set(event_target_value(&ev))
                            />
                        </div>
                        <Show when=move || kind_id.get().is_some() fallback=|| ()>
                            {move || {
                                let fields = kind_fields.get();
                                if fields.is_empty() {
                                    ().into_view()
                                } else {
                                    render_kind_fields_section(&fields, soft_field_map)
                                }
                            }}
                        </Show>
                        <Show when=move || error.get().is_some() fallback=|| ()>
                            <div class="error">
                                {move || error.get().unwrap_or_default()}
                            </div>
                        </Show>
                    </div>
                    <div class="modal-footer">
                        <button
                            class="btn btn-secondary"
                            prop:disabled=move || saving.get()
                            on:click=move |_| close_and_reset()
                        >
                            "Cancel"
                        </button>
                        <button
                            class="btn btn-primary"
                            style="width:auto;"
                            prop:disabled=move || saving.get()
                            on:click=move |_| {
                                if kind_id.get_untracked().is_none() {
                                    error.set(Some("Please select a type".to_string()));
                                    return;
                                }
                                if name.get_untracked().is_empty() {
                                    error.set(Some("Name is required".to_string()));
                                    return;
                                }
                                error.set(None);
                                saving.set(true);
                                save_action.dispatch(());
                            }
                        >
                            {move || if saving.get() { "Saving..." } else { "Save" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

fn render_kind_fields_section(
    kind_fields: &[KindFieldDef],
    soft_field_map: RwSignal<HashMap<String, serde_json::Value>>,
) -> View {
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
                    let fname = field_def.name.clone();
                    let label = field_def
                        .display_name
                        .clone()
                        .unwrap_or_else(|| format_field_name(&fname));
                    let ft = field_def.field_type.clone();
                    let enum_values = field_def.enum_values.clone();
                    view! {
                        <div class="form-group">
                            <label>{label}</label>
                            {render_soft_field_input(fname, ft, enum_values, soft_field_map)}
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_view()
}
