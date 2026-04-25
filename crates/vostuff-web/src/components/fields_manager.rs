use leptos::*;
use uuid::Uuid;

use crate::server_fns::fields::{Field, create_field, delete_field, get_fields, update_field};

// ── Top-level component ───────────────────────────────────────────────────────

#[component]
pub fn FieldsManager(org_id: Uuid) -> impl IntoView {
    let refresh = create_rw_signal(0u32);
    let fields_resource = create_resource(
        move || (org_id, refresh.get()),
        |(o, _)| async move { get_fields(o).await },
    );

    let show_create = create_rw_signal(false);
    let editing_field: RwSignal<Option<Field>> = create_rw_signal(None);

    view! {
        <div>
            <div style="margin-bottom: 16px;">
                <button class="btn btn-primary" on:click=move |_| show_create.set(true)>
                    "+ New Field"
                </button>
            </div>

            <Transition fallback=move || {
                view! { <div class="loading">"Loading fields..."</div> }
            }>
                {move || {
                    match fields_resource.get() {
                        Some(Ok(fields)) => {
                            let (shared, org_fields): (Vec<Field>, Vec<Field>) =
                                fields.into_iter().partition(|f| f.is_shared);
                            view! {
                                <div class="mgmt-section">
                                    <h3>"Shared Fields"</h3>
                                    {if shared.is_empty() {
                                        view! {
                                            <p style="color:#888;font-size:13px;">
                                                "No shared fields."
                                            </p>
                                        }
                                            .into_view()
                                    } else {
                                        shared
                                            .into_iter()
                                            .map(|f| {
                                                view! {
                                                    <FieldRow
                                                        field=f
                                                        org_id=org_id
                                                        on_refresh=Callback::new(move |_| {
                                                            refresh.update(|c| *c += 1)
                                                        })
                                                        on_edit=Callback::new(move |_: Field| {})
                                                    />
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </div>
                                <div class="mgmt-section">
                                    <h3>"Your Fields"</h3>
                                    {if org_fields.is_empty() {
                                        view! {
                                            <p style="color:#888;font-size:13px;">
                                                "No org fields yet."
                                            </p>
                                        }
                                            .into_view()
                                    } else {
                                        org_fields
                                            .into_iter()
                                            .map(|f| {
                                                let ef = editing_field;
                                                view! {
                                                    <FieldRow
                                                        field=f
                                                        org_id=org_id
                                                        on_refresh=Callback::new(move |_| {
                                                            refresh.update(|c| *c += 1)
                                                        })
                                                        on_edit=Callback::new(move |f: Field| {
                                                            ef.set(Some(f))
                                                        })
                                                    />
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </div>

                                <Show when=move || show_create.get() fallback=|| ()>
                                    <CreateFieldModal
                                        org_id=org_id
                                        on_close=Callback::new(move |_| show_create.set(false))
                                        on_created=Callback::new(move |_| {
                                            show_create.set(false);
                                            refresh.update(|c| *c += 1);
                                        })
                                    />
                                </Show>

                                <Show
                                    when=move || editing_field.get().is_some()
                                    fallback=|| ()
                                >
                                    {move || {
                                        editing_field
                                            .get()
                                            .map(|f| {
                                                view! {
                                                    <EditFieldModal
                                                        org_id=org_id
                                                        field=f
                                                        on_close=Callback::new(move |_| {
                                                            editing_field.set(None)
                                                        })
                                                        on_saved=Callback::new(move |_| {
                                                            editing_field.set(None);
                                                            refresh.update(|c| *c += 1);
                                                        })
                                                    />
                                                }
                                            })
                                    }}
                                </Show>
                            }
                                .into_view()
                        }
                        Some(Err(e)) => {
                            view! {
                                <div class="error">{format!("Error loading fields: {}", e)}</div>
                            }
                                .into_view()
                        }
                        None => view! { <div class="loading">"Loading..."</div> }.into_view(),
                    }
                }}
            </Transition>
        </div>
    }
}

// ── FieldRow ──────────────────────────────────────────────────────────────────

#[component]
fn FieldRow(
    field: Field,
    org_id: Uuid,
    on_refresh: Callback<()>,
    on_edit: Callback<Field>,
) -> impl IntoView {
    let is_shared = field.is_shared;
    let field_id = field.id;
    let field_name = field.name.clone();
    let field_type = field.field_type.clone();
    let display_name = field.display_name.clone().unwrap_or_default();

    let enum_summary = if field.field_type == "enum" && !field.enum_values.is_empty() {
        let vals: Vec<String> = field
            .enum_values
            .iter()
            .map(|ev| ev.display_value.clone().unwrap_or_else(|| ev.value.clone()))
            .collect();
        format!("({})", vals.join(", "))
    } else {
        String::new()
    };

    let row_error: RwSignal<Option<String>> = create_rw_signal(None);
    let field_for_edit = store_value(field.clone());

    let delete_action = create_action(move |_: &()| async move {
        delete_field(org_id, field_id).await
    });

    create_effect(move |_| {
        if let Some(result) = delete_action.value().get() {
            match result {
                Ok(_) => on_refresh.call(()),
                Err(e) => row_error.set(Some(e.to_string())),
            }
        }
    });

    view! {
        <div>
            <div class="mgmt-row">
                <span class="mgmt-row-name">{field_name}</span>
                <span class="mgmt-row-display">{display_name}</span>
                <span class="field-type-badge">{field_type}</span>
                <span class="mgmt-row-meta">{enum_summary}</span>
                {if is_shared {
                    view! { <span class="shared-badge">"shared"</span> }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }}
                {if !is_shared {
                    let ffe = field_for_edit;
                    view! {
                        <div class="mgmt-row-actions">
                            <button
                                class="btn btn-secondary btn-sm"
                                on:click=move |_| {
                                    row_error.set(None);
                                    on_edit.call(ffe.get_value());
                                }
                            >
                                "Edit"
                            </button>
                            <button
                                class="btn btn-danger btn-sm"
                                disabled=move || delete_action.pending().get()
                                on:click=move |_| {
                                    row_error.set(None);
                                    delete_action.dispatch(());
                                }
                            >
                                "Delete"
                            </button>
                        </div>
                    }
                        .into_view()
                } else {
                    view! { <div></div> }.into_view()
                }}
            </div>
            <Show when=move || row_error.get().is_some() fallback=|| ()>
                <div class="mgmt-row-error">
                    {move || row_error.get().unwrap_or_default()}
                </div>
            </Show>
        </div>
    }
}

// ── Enum value editor ─────────────────────────────────────────────────────────

/// (stable_key, value, display_value) triples.
/// The stable_key is used by <For> so that existing DOM nodes are not
/// recreated (and lose focus) when any row's content changes.
#[component]
fn EnumValueEditor(rows: RwSignal<Vec<(u32, String, String)>>) -> impl IntoView {
    // Start next_key above every key already in the list.
    let next_key = create_rw_signal(
        rows.get_untracked()
            .iter()
            .map(|(id, _, _)| *id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0),
    );

    // When set to Some(key), the row with that key should focus its first input.
    let focus_key: RwSignal<Option<u32>> = create_rw_signal(None);

    view! {
        <div>
            <For
                each=move || rows.get()
                key=|(id, _, _)| *id
                children=move |(id, val, dv)| {
                    let value_ref = create_node_ref::<html::Input>();

                    create_effect(move |_| {
                        if focus_key.get() == Some(id) {
                            if let Some(el) = value_ref.get() {
                                let _ = el.focus();
                                focus_key.set(None);
                            }
                        }
                    });

                    view! {
                        <div class="enum-val-row">
                            <input
                                node_ref=value_ref
                                type="text"
                                class="form-control"
                                placeholder="value"
                                prop:value=val
                                on:input=move |e| {
                                    rows.update(|r| {
                                        if let Some(row) = r.iter_mut().find(|(i, _, _)| *i == id) {
                                            row.1 = event_target_value(&e);
                                        }
                                    });
                                }
                            />
                            <input
                                type="text"
                                class="form-control"
                                placeholder="display value (optional)"
                                prop:value=dv
                                on:input=move |e| {
                                    rows.update(|r| {
                                        if let Some(row) = r.iter_mut().find(|(i, _, _)| *i == id) {
                                            row.2 = event_target_value(&e);
                                        }
                                    });
                                }
                            />
                            <button
                                class="btn btn-sm btn-danger"
                                on:click=move |_| {
                                    rows.update(|r| r.retain(|(i, _, _)| *i != id));
                                }
                            >
                                "Remove"
                            </button>
                        </div>
                    }
                }
            />
            <button
                class="btn btn-secondary btn-sm"
                on:click=move |_| {
                    let key = next_key.get_untracked();
                    next_key.update(|k| *k += 1);
                    rows.update(|r| r.push((key, String::new(), String::new())));
                    focus_key.set(Some(key));
                }
            >
                "+ Add Value"
            </button>
        </div>
    }
}

// ── CreateFieldModal ──────────────────────────────────────────────────────────

#[component]
fn CreateFieldModal(
    org_id: Uuid,
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let name = create_rw_signal(String::new());
    let display_name = create_rw_signal(String::new());
    let field_type = create_rw_signal("string".to_string());
    let enum_rows: RwSignal<Vec<(u32, String, String)>> = create_rw_signal(Vec::new());
    let saving = create_rw_signal(false);
    let error: RwSignal<Option<String>> = create_rw_signal(None);

    let save_action = create_action(move |_: &()| {
        let n = name.get_untracked();
        let dn = display_name.get_untracked();
        let ft = field_type.get_untracked();
        let rows = enum_rows.get_untracked();
        async move {
            let ev_json = if ft == "enum" {
                let ev: Vec<serde_json::Value> = rows
                    .into_iter()
                    .enumerate()
                    .map(|(i, (_, v, dv))| {
                        serde_json::json!({
                            "value": v,
                            "display_value": if dv.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(dv) },
                            "sort_order": i as i32,
                        })
                    })
                    .collect();
                Some(serde_json::to_string(&ev).unwrap_or_default())
            } else {
                None
            };
            create_field(
                org_id,
                n,
                if dn.is_empty() { None } else { Some(dn) },
                ft,
                ev_json,
            )
            .await
        }
    });

    create_effect(move |_| {
        saving.set(save_action.pending().get());
        if let Some(result) = save_action.value().get() {
            match result {
                Ok(_) => on_created.call(()),
                Err(e) => error.set(Some(e.to_string())),
            }
        }
    });

    view! {
        <div class="modal-overlay" on:click=move |_| on_close.call(())>
            <div
                class="modal"
                on:click=move |e| {
                    e.stop_propagation();
                }
            >
                <div class="modal-header">
                    <h2>"Create Field"</h2>
                </div>
                <div class="modal-body">
                    <div class="form-group">
                        <label>"Name"
                            <span style="font-size:11px;color:#888;margin-left:6px;">
                                "(cannot be changed after creation)"
                            </span>
                        </label>
                        <input
                            type="text"
                            class="form-control"
                            placeholder="e.g. release_year"
                            prop:value=move || name.get()
                            on:input=move |e| name.set(event_target_value(&e))
                        />
                    </div>
                    <div class="form-group">
                        <label>"Display Name"</label>
                        <input
                            type="text"
                            class="form-control"
                            placeholder="e.g. Release Year"
                            prop:value=move || display_name.get()
                            on:input=move |e| display_name.set(event_target_value(&e))
                        />
                    </div>
                    <div class="form-group">
                        <label>"Type"</label>
                        <select
                            class="form-control"
                            on:change=move |e| field_type.set(event_target_value(&e))
                        >
                            <option value="string">"String"</option>
                            <option value="text">"Text"</option>
                            <option value="number">"Number"</option>
                            <option value="boolean">"Boolean"</option>
                            <option value="date">"Date"</option>
                            <option value="datetime">"DateTime"</option>
                            <option value="enum">"Enum"</option>
                        </select>
                    </div>
                    <Show when=move || field_type.get() == "enum" fallback=|| ()>
                        <div class="form-group">
                            <label>"Enum Values"</label>
                            <EnumValueEditor rows=enum_rows/>
                        </div>
                    </Show>
                    <Show when=move || error.get().is_some() fallback=|| ()>
                        <div class="error">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" on:click=move |_| on_close.call(())>
                        "Cancel"
                    </button>
                    <button
                        class="btn btn-primary"
                        disabled=move || saving.get() || name.get().is_empty()
                        on:click=move |_| {
                            error.set(None);
                            save_action.dispatch(());
                        }
                    >
                        {move || if saving.get() { "Creating..." } else { "Create" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ── EditFieldModal ────────────────────────────────────────────────────────────

#[component]
fn EditFieldModal(
    org_id: Uuid,
    field: Field,
    on_close: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let field_id = field.id;
    let is_enum = field.field_type == "enum";
    let display_name = create_rw_signal(field.display_name.clone().unwrap_or_default());

    let initial_rows: Vec<(u32, String, String)> = field
        .enum_values
        .iter()
        .enumerate()
        .map(|(i, ev)| (i as u32, ev.value.clone(), ev.display_value.clone().unwrap_or_default()))
        .collect();
    let enum_rows: RwSignal<Vec<(u32, String, String)>> = create_rw_signal(initial_rows);

    let saving = create_rw_signal(false);
    let error: RwSignal<Option<String>> = create_rw_signal(None);

    let save_action = create_action(move |_: &()| {
        let dn = display_name.get_untracked();
        let rows = enum_rows.get_untracked();
        let is_enum2 = is_enum;
        async move {
            let ev_json = if is_enum2 {
                let ev: Vec<serde_json::Value> = rows
                    .into_iter()
                    .enumerate()
                    .map(|(i, (_, v, dv))| {
                        serde_json::json!({
                            "value": v,
                            "display_value": if dv.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(dv) },
                            "sort_order": i as i32,
                        })
                    })
                    .collect();
                Some(serde_json::to_string(&ev).unwrap_or_default())
            } else {
                None
            };
            update_field(
                org_id,
                field_id,
                if dn.is_empty() { None } else { Some(dn) },
                ev_json,
            )
            .await
        }
    });

    create_effect(move |_| {
        saving.set(save_action.pending().get());
        if let Some(result) = save_action.value().get() {
            match result {
                Ok(_) => on_saved.call(()),
                Err(e) => error.set(Some(e.to_string())),
            }
        }
    });

    view! {
        <div class="modal-overlay" on:click=move |_| on_close.call(())>
            <div
                class="modal"
                on:click=move |e| {
                    e.stop_propagation();
                }
            >
                <div class="modal-header">
                    <h2>"Edit Field"</h2>
                </div>
                <div class="modal-body">
                    <div class="form-group">
                        <label>"Display Name"</label>
                        <input
                            type="text"
                            class="form-control"
                            prop:value=move || display_name.get()
                            on:input=move |e| display_name.set(event_target_value(&e))
                        />
                    </div>
                    <Show when=move || is_enum fallback=|| ()>
                        <div class="form-group">
                            <label>"Enum Values"</label>
                            <EnumValueEditor rows=enum_rows/>
                        </div>
                    </Show>
                    <Show when=move || error.get().is_some() fallback=|| ()>
                        <div class="error">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" on:click=move |_| on_close.call(())>
                        "Cancel"
                    </button>
                    <button
                        class="btn btn-primary"
                        disabled=move || saving.get()
                        on:click=move |_| {
                            error.set(None);
                            save_action.dispatch(());
                        }
                    >
                        {move || if saving.get() { "Saving..." } else { "Save" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
