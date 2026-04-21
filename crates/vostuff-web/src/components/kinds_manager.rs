use leptos::*;
use uuid::Uuid;

use crate::server_fns::fields::{Field, get_fields};
use crate::server_fns::kinds::{
    Kind, create_kind, delete_kind, get_kinds_full, override_kind, revert_kind, update_kind,
};

// ── Top-level component ───────────────────────────────────────────────────────

#[component]
pub fn KindsManager(org_id: Uuid) -> impl IntoView {
    let refresh = create_rw_signal(0u32);
    let kinds_resource = create_resource(
        move || (org_id, refresh.get()),
        |(o, _)| async move { get_kinds_full(o).await },
    );
    let fields_resource = create_resource(
        move || org_id,
        |o| async move { get_fields(o).await },
    );

    let show_create = create_rw_signal(false);
    let editing_kind: RwSignal<Option<Kind>> = create_rw_signal(None);

    view! {
        <div>
            <div style="margin-bottom: 16px;">
                <button class="btn btn-primary" on:click=move |_| show_create.set(true)>
                    "+ New Kind"
                </button>
            </div>

            <Transition fallback=move || view! { <div class="loading">"Loading kinds..."</div> }>
                {move || {
                    match (kinds_resource.get(), fields_resource.get()) {
                        (Some(Ok(kinds)), Some(Ok(all_fields))) => {
                            let (shared, org_kinds): (Vec<Kind>, Vec<Kind>) =
                                kinds.into_iter().partition(|k| k.is_shared);
                            let shared_names: std::collections::HashSet<String> =
                                shared.iter().map(|k| k.name.clone()).collect();
                            let all_fields = store_value(all_fields);
                            view! {
                                <div class="mgmt-section">
                                    <h3>"Shared Kinds"</h3>
                                    {if shared.is_empty() {
                                        view! {
                                            <p style="color:#888;font-size:13px;">
                                                "No shared kinds."
                                            </p>
                                        }
                                            .into_view()
                                    } else {
                                        shared
                                            .into_iter()
                                            .map(|k| {
                                                view! {
                                                    <KindRow
                                                        kind=k
                                                        org_id=org_id
                                                        has_shared_counterpart=false
                                                        on_refresh=Callback::new(move |_| {
                                                            refresh.update(|c| *c += 1)
                                                        })
                                                        on_edit=Callback::new(move |_: Kind| {})
                                                    />
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </div>
                                <div class="mgmt-section">
                                    <h3>"Your Kinds"</h3>
                                    {if org_kinds.is_empty() {
                                        view! {
                                            <p style="color:#888;font-size:13px;">
                                                "No org kinds yet."
                                            </p>
                                        }
                                            .into_view()
                                    } else {
                                        org_kinds
                                            .into_iter()
                                            .map(|k| {
                                                let ek = editing_kind;
                                                let has_shared = shared_names.contains(&k.name);
                                                view! {
                                                    <KindRow
                                                        kind=k
                                                        org_id=org_id
                                                        has_shared_counterpart=has_shared
                                                        on_refresh=Callback::new(move |_| {
                                                            refresh.update(|c| *c += 1)
                                                        })
                                                        on_edit=Callback::new(move |k: Kind| {
                                                            ek.set(Some(k))
                                                        })
                                                    />
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </div>

                                <Show when=move || show_create.get() fallback=|| ()>
                                    <CreateKindModal
                                        org_id=org_id
                                        all_fields=all_fields.get_value()
                                        on_close=Callback::new(move |_| show_create.set(false))
                                        on_created=Callback::new(move |_| {
                                            show_create.set(false);
                                            refresh.update(|c| *c += 1);
                                        })
                                    />
                                </Show>

                                <Show
                                    when=move || editing_kind.get().is_some()
                                    fallback=|| ()
                                >
                                    {move || {
                                        editing_kind
                                            .get()
                                            .map(|k| {
                                                view! {
                                                    <EditKindModal
                                                        org_id=org_id
                                                        kind=k
                                                        all_fields=all_fields.get_value()
                                                        on_close=Callback::new(move |_| {
                                                            editing_kind.set(None)
                                                        })
                                                        on_saved=Callback::new(move |_| {
                                                            editing_kind.set(None);
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
                        (Some(Err(e)), _) | (_, Some(Err(e))) => {
                            view! {
                                <div class="error">{format!("Error loading data: {}", e)}</div>
                            }
                                .into_view()
                        }
                        _ => view! { <div class="loading">"Loading..."</div> }.into_view(),
                    }
                }}
            </Transition>
        </div>
    }
}

// ── KindRow ───────────────────────────────────────────────────────────────────

#[component]
fn KindRow(
    kind: Kind,
    org_id: Uuid,
    has_shared_counterpart: bool,
    on_refresh: Callback<()>,
    on_edit: Callback<Kind>,
) -> impl IntoView {
    let is_shared = kind.is_shared;
    let kind_id = kind.id;
    let kind_name = kind.name.clone();

    let row_error: RwSignal<Option<String>> = create_rw_signal(None);
    let revert_msg: RwSignal<Option<String>> = create_rw_signal(None);

    let kind_for_edit = store_value(kind.clone());

    let override_action = create_action(move |_: &()| {
        async move { override_kind(org_id, kind_id).await }
    });

    let delete_action = create_action(move |_: &()| async move {
        delete_kind(org_id, kind_id).await
    });

    let revert_action =
        create_action(move |_: &()| async move { revert_kind(org_id, kind_id).await });

    // Watch override_action result
    create_effect(move |_| {
        if let Some(result) = override_action.value().get() {
            match result {
                Ok(_) => on_refresh.call(()),
                Err(e) => row_error.set(Some(e.to_string())),
            }
        }
    });

    // Watch delete_action result
    create_effect(move |_| {
        if let Some(result) = delete_action.value().get() {
            match result {
                Ok(_) => on_refresh.call(()),
                Err(e) => row_error.set(Some(e.to_string())),
            }
        }
    });

    // Watch revert_action result
    create_effect(move |_| {
        if let Some(result) = revert_action.value().get() {
            match result {
                Ok(rv) => {
                    let msg = format!(
                        "Reverted — {} items reassigned",
                        rv.items_reassigned
                    );
                    revert_msg.set(Some(msg));
                    on_refresh.call(());
                }
                Err(e) => row_error.set(Some(e.to_string())),
            }
        }
    });

    let field_chips = kind
        .fields
        .iter()
        .map(|f| {
            let label = f
                .display_name
                .clone()
                .unwrap_or_else(|| f.name.clone());
            view! { <span class="kind-field-chip">{label}</span> }
        })
        .collect_view();

    let display = kind
        .display_name
        .clone()
        .unwrap_or_default();

    view! {
        <div>
            <div class="mgmt-row">
                <span class="mgmt-row-name">{kind_name.clone()}</span>
                <span class="mgmt-row-display">{display}</span>
                <span class="mgmt-row-meta">{field_chips}</span>
                {if is_shared {
                    view! {
                        <span class="shared-badge">"shared"</span>
                    }
                        .into_view()
                } else {
                    view! { <span></span> }.into_view()
                }}
                <div class="mgmt-row-actions">
                    {if is_shared {
                        view! {
                            <button
                                class="btn btn-secondary btn-sm"
                                disabled=move || override_action.pending().get()
                                on:click=move |_| {
                                    row_error.set(None);
                                    override_action.dispatch(());
                                }
                            >
                                "Override"
                            </button>
                        }
                            .into_view()
                    } else {
                        let kfe = kind_for_edit;
                        view! {
                            <button
                                class="btn btn-secondary btn-sm"
                                on:click=move |_| {
                                    row_error.set(None);
                                    on_edit.call(kfe.get_value());
                                }
                            >
                                "Edit"
                            </button>
                            <Show when=move || has_shared_counterpart fallback=|| ()>
                                <button
                                    class="btn btn-secondary btn-sm"
                                    disabled=move || revert_action.pending().get()
                                    on:click=move |_| {
                                        row_error.set(None);
                                        revert_action.dispatch(());
                                    }
                                >
                                    "Revert"
                                </button>
                            </Show>
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
                        }
                            .into_view()
                    }}
                </div>
            </div>
            <Show when=move || row_error.get().is_some() fallback=|| ()>
                <div class="mgmt-row-error">
                    {move || row_error.get().unwrap_or_default()}
                </div>
            </Show>
            <Show when=move || revert_msg.get().is_some() fallback=|| ()>
                <div style="color:#2c7a3e;font-size:12px;margin-top:2px;">
                    {move || revert_msg.get().unwrap_or_default()}
                </div>
            </Show>
        </div>
    }
}

// ── Field picker helper ───────────────────────────────────────────────────────

/// Two-column field picker: available (left) ↔ selected in order (right)
#[component]
fn FieldPicker(
    all_fields: Vec<Field>,
    ordered_ids: RwSignal<Vec<Uuid>>,
) -> impl IntoView {
    let all = store_value(all_fields);

    // Helpers to get field name by id
    let field_label = move |id: Uuid| -> String {
        all.get_value()
            .iter()
            .find(|f| f.id == id)
            .map(|f| {
                f.display_name
                    .clone()
                    .unwrap_or_else(|| f.name.clone())
            })
            .unwrap_or_else(|| id.to_string())
    };

    view! {
        <div class="field-picker">
            // Left: available (not yet selected)
            <div>
                <div style="font-size:12px;font-weight:600;margin-bottom:4px;color:#555;">
                    "Available fields"
                </div>
                <div class="field-picker-available">
                    {move || {
                        let selected = ordered_ids.get();
                        all.get_value()
                            .into_iter()
                            .filter(|f| !selected.contains(&f.id))
                            .map(|f| {
                                let fid = f.id;
                                let label = f
                                    .display_name
                                    .clone()
                                    .unwrap_or_else(|| f.name.clone());
                                view! {
                                    <div
                                        class="field-picker-item"
                                        on:click=move |_| {
                                            ordered_ids.update(|v| v.push(fid));
                                        }
                                    >
                                        {label}
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
            // Right: selected in order
            <div>
                <div style="font-size:12px;font-weight:600;margin-bottom:4px;color:#555;">
                    "Selected fields (in order)"
                </div>
                <div class="field-picker-selected">
                    {move || {
                        let ids = ordered_ids.get();
                        ids.iter()
                            .cloned()
                            .enumerate()
                            .map(|(i, fid)| {
                                let at_end = i + 1 >= ids.len();
                                let label = field_label(fid);
                                view! {
                                    <div class="field-picker-selected-row">
                                        <span>{label}</span>
                                        <button
                                            class="btn btn-sm btn-reorder"
                                            disabled=move || i == 0
                                            on:click=move |_| {
                                                ordered_ids
                                                    .update(|v| {
                                                        if i > 0 {
                                                            v.swap(i - 1, i);
                                                        }
                                                    });
                                            }
                                        >
                                            "↑"
                                        </button>
                                        <button
                                            class="btn btn-sm btn-reorder"
                                            disabled=at_end
                                            on:click=move |_| {
                                                ordered_ids
                                                    .update(|v| {
                                                        if i + 1 < v.len() {
                                                            v.swap(i, i + 1);
                                                        }
                                                    });
                                            }
                                        >
                                            "↓"
                                        </button>
                                        <button
                                            class="btn btn-sm btn-danger"
                                            on:click=move |_| {
                                                ordered_ids
                                                    .update(|v| {
                                                        v.retain(|&x| x != fid);
                                                    });
                                            }
                                        >
                                            "×"
                                        </button>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
        </div>
    }
}

// ── CreateKindModal ───────────────────────────────────────────────────────────

#[component]
fn CreateKindModal(
    org_id: Uuid,
    all_fields: Vec<Field>,
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let name = create_rw_signal(String::new());
    let display_name = create_rw_signal(String::new());
    let ordered_ids: RwSignal<Vec<Uuid>> = create_rw_signal(Vec::new());
    let saving = create_rw_signal(false);
    let error: RwSignal<Option<String>> = create_rw_signal(None);

    let save_action = create_action(move |_: &()| {
        let n = name.get_untracked();
        let dn = display_name.get_untracked();
        let ids = ordered_ids.get_untracked();
        async move {
            let ids_json = serde_json::to_string(&ids)
                .map_err(|e| leptos::server_fn::error::ServerFnError::<leptos::server_fn::error::NoCustomError>::ServerError(e.to_string()))?;
            create_kind(
                org_id,
                n,
                if dn.is_empty() { None } else { Some(dn) },
                ids_json,
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
                    <h2>"Create Kind"</h2>
                </div>
                <div class="modal-body">
                    <div class="form-group">
                        <label>"Name"</label>
                        <input
                            type="text"
                            class="form-control"
                            placeholder="e.g. vinyl_record"
                            prop:value=move || name.get()
                            on:input=move |e| name.set(event_target_value(&e))
                        />
                    </div>
                    <div class="form-group">
                        <label>"Display Name"</label>
                        <input
                            type="text"
                            class="form-control"
                            placeholder="e.g. Vinyl Record"
                            prop:value=move || display_name.get()
                            on:input=move |e| display_name.set(event_target_value(&e))
                        />
                    </div>
                    <div class="form-group">
                        <label>"Fields"</label>
                        <FieldPicker all_fields=all_fields ordered_ids=ordered_ids/>
                    </div>
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

// ── EditKindModal ─────────────────────────────────────────────────────────────

#[component]
fn EditKindModal(
    org_id: Uuid,
    kind: Kind,
    all_fields: Vec<Field>,
    on_close: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let kind_id = kind.id;
    let display_name = create_rw_signal(kind.display_name.clone().unwrap_or_default());
    let initial_ids: Vec<Uuid> = kind.fields.iter().map(|f| f.id).collect();
    let ordered_ids: RwSignal<Vec<Uuid>> = create_rw_signal(initial_ids);

    let saving = create_rw_signal(false);
    let error: RwSignal<Option<String>> = create_rw_signal(None);
    let show_force_warning = create_rw_signal(false);

    let do_save = move |force: bool| {
        let dn = display_name.get_untracked();
        let ids = ordered_ids.get_untracked();
        let ids_json = match serde_json::to_string(&ids) {
            Ok(j) => j,
            Err(e) => {
                error.set(Some(e.to_string()));
                return;
            }
        };
        let save_action = create_action(move |_: &()| {
            let dn2 = if dn.is_empty() { None } else { Some(dn.clone()) };
            let j = ids_json.clone();
            async move {
                update_kind(org_id, kind_id, dn2, Some(j), force).await
            }
        });
        create_effect(move |_| {
            saving.set(save_action.pending().get());
            if let Some(result) = save_action.value().get() {
                match result {
                    Ok(_) => on_saved.call(()),
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("data_loss_required") {
                            show_force_warning.set(true);
                        } else {
                            error.set(Some(msg));
                        }
                    }
                }
            }
        });
        save_action.dispatch(());
    };

    view! {
        <div class="modal-overlay" on:click=move |_| on_close.call(())>
            <div
                class="modal"
                on:click=move |e| {
                    e.stop_propagation();
                }
            >
                <div class="modal-header">
                    <h2>"Edit Kind"</h2>
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
                    <div class="form-group">
                        <label>"Fields"</label>
                        <FieldPicker all_fields=all_fields ordered_ids=ordered_ids/>
                    </div>
                    <Show when=move || show_force_warning.get() fallback=|| ()>
                        <div class="data-loss-warning">
                            <strong>"Warning: Data Loss"</strong>
                            "Removing these fields will delete existing field data from items. "
                            "Click Force Save to proceed."
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
                    <Show when=move || show_force_warning.get() fallback=|| ()>
                        <button
                            class="btn btn-danger"
                            disabled=move || saving.get()
                            on:click=move |_| {
                                error.set(None);
                                do_save(true);
                            }
                        >
                            "Force Save"
                        </button>
                    </Show>
                    <Show when=move || !show_force_warning.get() fallback=|| ()>
                        <button
                            class="btn btn-primary"
                            disabled=move || saving.get()
                            on:click=move |_| {
                                error.set(None);
                                show_force_warning.set(false);
                                do_save(false);
                            }
                        >
                            {move || if saving.get() { "Saving..." } else { "Save" }}
                        </button>
                    </Show>
                </div>
            </div>
        </div>
    }
}
