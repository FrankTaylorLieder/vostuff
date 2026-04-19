use leptos::*;
use std::collections::HashMap;

use crate::server_fns::kinds::KindEnumValue;

pub fn value_to_edit_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => {
            // Show integers without a decimal point
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                f.to_string()
            } else {
                n.to_string()
            }
        }
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        _ => v.to_string(),
    }
}

pub fn format_field_name(name: &str) -> String {
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

pub fn format_soft_field_value(
    field_type: &str,
    raw: &serde_json::Value,
    enum_values: &[KindEnumValue],
) -> String {
    let s = match raw {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                f.to_string()
            } else {
                n.to_string()
            }
        }
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    };
    match field_type {
        "boolean" => {
            if s == "true" {
                "Yes".to_string()
            } else if s == "false" {
                "No".to_string()
            } else {
                s
            }
        }
        "enum" => enum_values
            .iter()
            .find(|ev| ev.value == s)
            .and_then(|ev| ev.display_value.clone())
            .unwrap_or(s),
        _ => s,
    }
}

/// Render a type-specific input for a single soft field.
/// The map stores serde_json::Value so types are preserved through edit/save.
pub fn render_soft_field_input(
    name: String,
    field_type: String,
    enum_values: Vec<KindEnumValue>,
    soft_field_map: RwSignal<HashMap<String, serde_json::Value>>,
) -> View {
    let n1 = name.clone();
    // Convert the stored Value to a display string for HTML inputs
    let current_val =
        move || soft_field_map.with(|m| m.get(&n1).map(|v| value_to_edit_str(v)).unwrap_or_default());

    match field_type.as_str() {
        "boolean" => {
            let n = name.clone();
            view! {
                <input
                    type="checkbox"
                    class="edit-checkbox"
                    prop:checked=move || current_val() == "true"
                    on:change=move |ev| {
                        let checked = event_target_checked(&ev);
                        soft_field_map.update(|m| {
                            m.insert(n.clone(), serde_json::Value::Bool(checked));
                        });
                    }
                />
            }
            .into_view()
        }
        "number" => {
            let n = name.clone();
            view! {
                <input
                    type="number"
                    class="edit-input"
                    prop:value=current_val
                    on:input=move |ev| {
                        let s = event_target_value(&ev);
                        let v = if s.is_empty() {
                            serde_json::Value::Null
                        } else if let Ok(i) = s.parse::<i64>() {
                            serde_json::json!(i)
                        } else if let Ok(f) = s.parse::<f64>() {
                            serde_json::json!(f)
                        } else {
                            serde_json::Value::String(s)
                        };
                        soft_field_map.update(|m| {
                            m.insert(n.clone(), v);
                        });
                    }
                />
            }
            .into_view()
        }
        "date" => {
            let n = name.clone();
            view! {
                <input
                    type="date"
                    class="edit-input"
                    prop:value=current_val
                    on:input=move |ev| {
                        soft_field_map.update(|m| {
                            m.insert(n.clone(), serde_json::Value::String(event_target_value(&ev)));
                        });
                    }
                />
            }
            .into_view()
        }
        "datetime" => {
            let n = name.clone();
            view! {
                <input
                    type="datetime-local"
                    class="edit-input"
                    prop:value=current_val
                    on:input=move |ev| {
                        soft_field_map.update(|m| {
                            m.insert(n.clone(), serde_json::Value::String(event_target_value(&ev)));
                        });
                    }
                />
            }
            .into_view()
        }
        "text" => {
            let n = name.clone();
            view! {
                <textarea
                    class="edit-textarea"
                    prop:value=current_val
                    on:input=move |ev| {
                        soft_field_map.update(|m| {
                            m.insert(n.clone(), serde_json::Value::String(event_target_value(&ev)));
                        });
                    }
                />
            }
            .into_view()
        }
        "enum" => {
            let n = name.clone();
            view! {
                <select
                    class="edit-select"
                    prop:value=current_val
                    on:change=move |ev| {
                        soft_field_map.update(|m| {
                            m.insert(n.clone(), serde_json::Value::String(event_target_value(&ev)));
                        });
                    }
                >
                    <option value="">"- Select -"</option>
                    {enum_values
                        .iter()
                        .map(|e| {
                            let val = e.value.clone();
                            let label = e.display_value.clone().unwrap_or_else(|| val.clone());
                            view! { <option value=val>{label}</option> }
                        })
                        .collect_view()}
                </select>
            }
            .into_view()
        }
        _ => {
            // "string" and anything unrecognised → text input
            let n = name.clone();
            view! {
                <input
                    type="text"
                    class="edit-input"
                    prop:value=current_val
                    on:input=move |ev| {
                        soft_field_map.update(|m| {
                            m.insert(n.clone(), serde_json::Value::String(event_target_value(&ev)));
                        });
                    }
                />
            }
            .into_view()
        }
    }
}
