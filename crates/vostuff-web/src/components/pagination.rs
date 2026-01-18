use leptos::*;

#[component]
pub fn Pagination(
    current_page: ReadSignal<i64>,
    total_pages: i64,
    total_items: i64,
    per_page: ReadSignal<i64>,
    set_page: WriteSignal<i64>,
    set_per_page: WriteSignal<i64>,
) -> impl IntoView {
    let page_sizes = [10, 25, 50, 100];

    let start_item = move || {
        let page = current_page.get();
        let pp = per_page.get();
        if total_items == 0 {
            0
        } else {
            (page - 1) * pp + 1
        }
    };

    let end_item = move || {
        let page = current_page.get();
        let pp = per_page.get();
        std::cmp::min(page * pp, total_items)
    };

    let can_go_prev = move || current_page.get() > 1;
    let can_go_next = move || current_page.get() < total_pages;

    view! {
        <div class="pagination">
            <div class="pagination-info">
                <span>
                    {move || format!("{}-{} of {} items", start_item(), end_item(), total_items)}
                </span>
            </div>

            <div class="pagination-controls">
                <button
                    class="btn btn-secondary pagination-btn"
                    disabled=move || !can_go_prev()
                    on:click=move |_| {
                        if can_go_prev() {
                            set_page.update(|p| *p -= 1);
                        }
                    }
                >
                    "Previous"
                </button>

                <span class="pagination-page">
                    {move || format!("Page {} of {}", current_page.get(), total_pages.max(1))}
                </span>

                <button
                    class="btn btn-secondary pagination-btn"
                    disabled=move || !can_go_next()
                    on:click=move |_| {
                        if can_go_next() {
                            set_page.update(|p| *p += 1);
                        }
                    }
                >
                    "Next"
                </button>
            </div>

            <div class="pagination-size">
                <label for="page-size">"Items per page: "</label>
                <select
                    id="page-size"
                    on:change=move |ev| {
                        let value = event_target_value(&ev).parse::<i64>().unwrap_or(25);
                        set_per_page.set(value);
                        set_page.set(1); // Reset to first page when changing page size
                    }
                >
                    {page_sizes
                        .iter()
                        .map(|&size| {
                            view! {
                                <option
                                    value=size.to_string()
                                    selected=move || per_page.get() == size
                                >
                                    {size}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </div>
        </div>
    }
}
