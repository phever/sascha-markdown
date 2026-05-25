use gtk4 as gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ui::AppState;
use crate::ui::markup::apply_markup;

pub fn refresh_toolbar(state: Rc<RefCell<AppState>>, width: Option<i32>) {
    let s = state.borrow();
    let toolbar = match &s.toolbar {
        Some(t) => t,
        None => return,
    };

    // Clear existing children
    while let Some(child) = toolbar.first_child() {
        toolbar.remove(&child);
    }

    let active_tab = match s.get_active_tab() {
        Some(tab) => tab,
        None => return, // No active tab, so toolbar remains empty
    };

    let buffer = active_tab.borrow().buffer.clone();

    let config = s.config.clone();
    let all = config.formatters.all_formatters();
    let visible_items: Vec<_> = all.into_iter().filter(|(_, _, v, _)| *v).collect();
    
    // Calculate how many buttons fit
    // Average button width is ~38px, plus spacing. 
    // We leave some margin and room for the overflow button.
    let available_width = width.unwrap_or(800);
    let button_width_estimate = 42; 
    let primary_count = ((available_width - 50) / button_width_estimate).max(1) as usize;

    let (primary, overflow) = if visible_items.len() > primary_count {
        visible_items.split_at(primary_count)
    } else {
        (visible_items.as_slice(), [].as_slice())
    };

    let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    toolbar.append(&main_box);

    for (name, symbol, _, icon_name) in primary {
        let btn = gtk::Button::from_icon_name(icon_name);
        btn.set_tooltip_text(Some(name));
        let buffer_clone = buffer.clone();
        let symbol_clone = symbol.clone();
        btn.connect_clicked(move |_| {
            apply_markup(&buffer_clone, &symbol_clone);
        });
        main_box.append(&btn);
    }

    if !overflow.is_empty() {
        let menu_btn = gtk::MenuButton::new();
        menu_btn.set_icon_name("view-more-symbolic");
        menu_btn.set_tooltip_text(Some("More formatting"));
        
        let popover = gtk::Popover::new();
        let overflow_box = gtk::FlowBox::new();
        overflow_box.set_max_children_per_line(5);
        overflow_box.set_selection_mode(gtk::SelectionMode::None);
        overflow_box.set_margin_start(10);
        overflow_box.set_margin_end(10);
        overflow_box.set_margin_top(10);
        overflow_box.set_margin_bottom(10);
        overflow_box.set_column_spacing(5);
        overflow_box.set_row_spacing(5);

        for (name, symbol, _, icon_name) in overflow {
            let btn = gtk::Button::from_icon_name(icon_name);
            btn.set_tooltip_text(Some(name));
            let buffer_clone = buffer.clone();
            let symbol_clone = symbol.clone();
            let pop_clone = popover.clone();
            btn.connect_clicked(move |_| {
                apply_markup(&buffer_clone, &symbol_clone);
                pop_clone.popdown();
            });
            overflow_box.insert(&btn, -1);
        }

        popover.set_child(Some(&overflow_box));
        menu_btn.set_popover(Some(&popover));
        main_box.append(&menu_btn);
    }
}