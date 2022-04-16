// MyCitadel desktop wallet: bitcoin & RGB wallet based on GTK framework.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@pandoraprime.ch>
//
// Copyright (C) 2022 by Pandora Prime Sarl, Switzerland.
//
// This software is distributed without any warranty. You should have received
// a copy of the AGPL-3.0 License along with this software. If not, see
// <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use gladis::Gladis;
use gtk::prelude::*;
use gtk::{
    glib, Adjustment, Calendar, Label, ListBoxRow, Menu, MenuButton, RadioMenuItem, SpinButton,
};
use relm::StreamHandle;

use super::Condition;
use crate::view::settings;

#[derive(Clone, Gladis)]
pub struct RowWidgets {
    spending_row: ListBoxRow,
    sig_lbl: Label,
    sigs_menu: Menu,
    sigs_all_item: RadioMenuItem,
    sigs_atleast_item: RadioMenuItem,
    sigs_any_item: RadioMenuItem,
    sigs_spin: SpinButton,
    sigs_adj: Adjustment,
    sigtext_lbl: Label,
    lock_mbt: MenuButton,
    lock_menu: Menu,
    lock_anytime_item: RadioMenuItem,
    lock_after_item: RadioMenuItem,
    lock_older_item: RadioMenuItem,
    lock_lbl: Label,
    date_spin: SpinButton,
    date_adj: Adjustment,
    period_mbt: MenuButton,
    period_menu: Menu,
    period_days_item: RadioMenuItem,
    period_weeks_item: RadioMenuItem,
    period_months_item: RadioMenuItem,
    period_years_item: RadioMenuItem,
    period_lbl: Label,
    calendar_mbt: MenuButton,
    calendar_lbl: Label,
    calendar: Calendar,
}

impl RowWidgets {
    pub fn init(stream_: StreamHandle<settings::Msg>, item: &glib::Object) -> gtk::Widget {
        let glade_src = include_str!("spending_row.glade");
        let row_widgets = RowWidgets::from_string(glade_src).expect("glade file broken");

        let condition = item
            .downcast_ref::<Condition>()
            .expect("Row data is of wrong type");
        row_widgets.bind_model(condition);

        let stream = stream_.clone();
        // We use hack re-utilizing `can-default` property, since updates to `active` property are
        // not working in GTK3
        let toggle = move |mi: &RadioMenuItem| {
            mi.set_property("can-default", mi.is_active());
            stream.emit(settings::Msg::SpendingConditionChange);
        };

        row_widgets.sigs_all_item.connect_toggled(toggle.clone());
        row_widgets.sigs_any_item.connect_toggled(toggle.clone());
        row_widgets
            .sigs_atleast_item
            .connect_toggled(toggle.clone());
        row_widgets.sigs_all_item.set_property("can-default", true);

        row_widgets
            .lock_anytime_item
            .connect_toggled(toggle.clone());
        row_widgets.lock_after_item.connect_toggled(toggle.clone());
        row_widgets.lock_older_item.connect_toggled(toggle);
        row_widgets
            .lock_anytime_item
            .set_property("can-default", true);

        row_widgets.spending_row.upcast::<gtk::Widget>()
    }

    fn bind_model(&self, condition: &Condition) {
        let flags_ro = glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE;
        let flags_all = glib::BindingFlags::DEFAULT
            | glib::BindingFlags::SYNC_CREATE
            | glib::BindingFlags::BIDIRECTIONAL;
        self.sigs_all_item
            .bind_property("can-default", condition, "sigs-all")
            .flags(flags_ro)
            .build();
        self.sigs_any_item
            .bind_property("can-default", condition, "sigs-any")
            .flags(flags_ro)
            .build();
        self.sigs_atleast_item
            .bind_property("can-default", condition, "sigs-at-least")
            .flags(flags_ro)
            .build();
        condition
            .bind_property("sigs-no", &self.sigs_adj, "value")
            .flags(flags_all)
            .build();
        condition
            .bind_property("sigs-at-least", &self.sigs_spin, "visible")
            .flags(flags_ro)
            .build();
        condition
            .bind_property("sigs-at-least", &self.sigtext_lbl, "visible")
            .flags(flags_ro)
            .build();
        condition
            .bind_property("sigs-all", &self.sig_lbl, "label")
            .flags(flags_ro)
            .transform_to(|_, val| {
                if val.get().unwrap() {
                    Some("All signatures".to_value())
                } else {
                    None
                }
            })
            .build();
        condition
            .bind_property("sigs-any", &self.sig_lbl, "label")
            .flags(flags_ro)
            .transform_to(|_, val| {
                if val.get().unwrap() {
                    Some("Any signature".to_value())
                } else {
                    None
                }
            })
            .build();
        condition
            .bind_property("sigs-at-least", &self.sig_lbl, "label")
            .flags(flags_ro)
            .transform_to(|_, val| {
                if val.get().unwrap() {
                    Some("At least".to_value())
                } else {
                    None
                }
            })
            .build();

        self.lock_anytime_item
            .bind_property("can-default", condition, "lock-none")
            .flags(flags_ro)
            .build();
        self.lock_after_item
            .bind_property("can-default", condition, "lock-after")
            .flags(flags_ro)
            .build();
        self.lock_older_item
            .bind_property("can-default", condition, "lock-older")
            .flags(flags_ro)
            .build();
        condition
            .bind_property("lock-older", &self.calendar_mbt, "visible")
            .flags(flags_ro)
            .build();
        condition
            .bind_property("lock-after", &self.date_spin, "visible")
            .flags(flags_ro)
            .build();
        condition
            .bind_property("lock-after", &self.period_mbt, "visible")
            .flags(flags_ro)
            .build();
        condition
            .bind_property("lock-none", &self.lock_lbl, "label")
            .flags(flags_ro)
            .transform_to(|_, val| {
                if val.get().unwrap() {
                    Some("at any time".to_value())
                } else {
                    None
                }
            })
            .build();
        condition
            .bind_property("lock-after", &self.lock_lbl, "label")
            .flags(flags_ro)
            .transform_to(|_, val| {
                if val.get().unwrap() {
                    Some("after".to_value())
                } else {
                    None
                }
            })
            .build();
        condition
            .bind_property("lock-older", &self.lock_lbl, "label")
            .flags(flags_ro)
            .transform_to(|_, val| {
                if val.get().unwrap() {
                    Some("starting from".to_value())
                } else {
                    None
                }
            })
            .build();
    }
}
