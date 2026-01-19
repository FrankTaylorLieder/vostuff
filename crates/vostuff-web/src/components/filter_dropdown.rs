use leptos::*;
use std::collections::HashSet;

/// A single filter option with a value and display label
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
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
    let (is_open, set_is_open) = create_signal(false);

    // Store options and values for use in reactive closures
    let options_store = store_value(options.clone());
    let all_values = store_value(options.iter().map(|o| o.value.clone()).collect::<Vec<_>>());

    // Toggle a single option
    let toggle_option = move |value: String| {
        set_selected.update(|s| {
            if s.contains(&value) {
                s.remove(&value);
            } else {
                s.insert(value);
            }
        });
    };

    // Generate button text based on selection
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
                on:click=move |_| set_is_open.update(|o| *o = !*o)
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
                                set_selected
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
                                set_selected.update(|s| s.clear());
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
                                                checked=move || selected.get().contains(&value_for_check)
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
                        <button class="filter-done-btn" on:click=move |_| set_is_open.set(false)>
                            "Done"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Filter bar containing multiple filter dropdowns
#[component]
pub fn FilterBar(children: Children) -> impl IntoView {
    view! {
        <div class="filter-bar">
            {children()}
        </div>
    }
}
