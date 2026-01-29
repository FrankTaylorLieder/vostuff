use leptos::*;
use std::collections::HashSet;

/// A single filter option with a value and display label
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
}

/// Shared context so only one dropdown is open at a time.
/// Holds the label of the currently-open dropdown (None = all closed).
#[derive(Clone, Copy)]
pub struct ActiveDropdown {
    pub active: ReadSignal<Option<String>>,
    pub set_active: WriteSignal<Option<String>>,
}

/// Multi-select filter dropdown component
#[component]
pub fn FilterDropdown(
    /// Label shown on the dropdown button
    #[prop(into)]
    label: String,
    /// Available options to select from
    options: Vec<FilterOption>,
    /// Currently selected values
    selected: ReadSignal<HashSet<String>>,
    /// Callback when selection changes
    set_selected: WriteSignal<HashSet<String>>,
) -> impl IntoView {
    let dropdown_id = label.clone();
    let dropdown_id_for_open = dropdown_id.clone();
    let dropdown_id_for_done = dropdown_id.clone();

    // Use shared active-dropdown context if available, otherwise local signal
    let active_ctx = use_context::<ActiveDropdown>();

    let (local_open, set_local_open) = create_signal(false);

    let is_open = {
        let id = dropdown_id.clone();
        Signal::derive(move || match active_ctx {
            Some(ctx) => ctx.active.get().as_deref() == Some(&id),
            None => local_open.get(),
        })
    };

    // Local staging signal — edits happen here, committed on "Done"
    let (staged, set_staged) = create_signal::<HashSet<String>>(selected.get_untracked());

    // Sync staged from parent when the dropdown opens
    create_effect(move |_prev_open: Option<bool>| {
        let now_open = is_open.get();
        // Reset staged to committed selection on open (discard uncommitted changes)
        if now_open {
            set_staged.set(selected.get_untracked());
        }
        now_open
    });

    // Store options and values for use in reactive closures
    let options_store = store_value(options.clone());
    let all_values = store_value(options.iter().map(|o| o.value.clone()).collect::<Vec<_>>());

    // Toggle a single option in the staging signal
    let toggle_option = move |value: String| {
        set_staged.update(|s| {
            if s.contains(&value) {
                s.remove(&value);
            } else {
                s.insert(value);
            }
        });
    };

    // Generate button text based on committed selection
    let button_text = {
        let label = label.clone();
        move || {
            let sel = selected.get();
            let opts = options_store.get_value();
            if sel.is_empty() {
                format!("{}: All", label)
            } else if sel.len() == 1 {
                let value = sel.iter().next().unwrap();
                let display = opts
                    .iter()
                    .find(|o| &o.value == value)
                    .map(|o| o.label.clone())
                    .unwrap_or_else(|| value.clone());
                format!("{}: {}", label, display)
            } else if sel.len() == opts.len() {
                format!("{}: All", label)
            } else {
                format!("{}: {} selected", label, sel.len())
            }
        }
    };

    view! {
        <div class="filter-dropdown">
            <button
                class="filter-dropdown-btn"
                class:active=move || !selected.get().is_empty()
                on:click={
                    let id = dropdown_id_for_open.clone();
                    move |_| {
                        if let Some(ctx) = active_ctx {
                            let current = ctx.active.get_untracked();
                            if current.as_deref() == Some(&id) {
                                ctx.set_active.set(None);
                            } else {
                                ctx.set_active.set(Some(id.clone()));
                            }
                        } else {
                            set_local_open.update(|o| *o = !*o);
                        }
                    }
                }
            >
                <span class="filter-dropdown-text">{button_text}</span>
                <span class="filter-dropdown-arrow">
                    {move || if is_open.get() { "▲" } else { "▼" }}
                </span>
            </button>

            <Show when=move || is_open.get() fallback=|| ()>
                <div class="filter-dropdown-menu">
                    <div class="filter-dropdown-actions">
                        <button
                            class="filter-action-btn"
                            on:click=move |_| {
                                let values = all_values.get_value();
                                set_staged
                                    .update(|s| {
                                        for val in values {
                                            s.insert(val);
                                        }
                                    });
                            }
                        >

                            "Select All"
                        </button>
                        <button
                            class="filter-action-btn"
                            on:click=move |_| {
                                set_staged.update(|s| s.clear());
                            }
                        >

                            "Clear"
                        </button>
                    </div>
                    <div class="filter-dropdown-options">
                        {move || {
                            options_store
                                .get_value()
                                .into_iter()
                                .map(|opt| {
                                    let value_for_check = opt.value.clone();
                                    let value_for_toggle = opt.value.clone();
                                    let label = opt.label.clone();
                                    view! {
                                        <label class="filter-option">
                                            <input
                                                type="checkbox"
                                                checked=move || staged.get().contains(&value_for_check)
                                                on:change=move |_| toggle_option(value_for_toggle.clone())
                                            />
                                            <span class="filter-option-label">{label}</span>
                                        </label>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                    <div class="filter-dropdown-footer">
                        <button
                            class="filter-done-btn"
                            on:click={
                                let id = dropdown_id_for_done.clone();
                                move |_| {
                                    set_selected.set(staged.get_untracked());
                                    if let Some(ctx) = active_ctx {
                                        let current = ctx.active.get_untracked();
                                        if current.as_deref() == Some(&id) {
                                            ctx.set_active.set(None);
                                        }
                                    } else {
                                        set_local_open.set(false);
                                    }
                                }
                            }
                        >
                            "Done"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Text search input that submits on Enter
#[component]
pub fn FilterSearchInput(
    /// Current input value
    value: ReadSignal<String>,
    /// Setter for the input value (updated on every keystroke)
    set_value: WriteSignal<String>,
    /// Setter for the committed search (updated on Enter)
    set_committed: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <input
            type="text"
            class="filter-search-input"
            placeholder="Search... (Enter to submit)"
            prop:value=move || value.get()
            on:input=move |ev| {
                set_value.set(event_target_value(&ev));
            }
            on:keydown=move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Enter" {
                    ev.prevent_default();
                    set_committed.set(value.get_untracked());
                }
            }
        />
    }
}

/// Filter bar containing multiple filter dropdowns
#[component]
pub fn FilterBar(children: Children) -> impl IntoView {
    let (active, set_active) = create_signal::<Option<String>>(None);
    provide_context(ActiveDropdown { active, set_active });

    view! {
        <div class="filter-bar">
            {children()}
        </div>
    }
}
