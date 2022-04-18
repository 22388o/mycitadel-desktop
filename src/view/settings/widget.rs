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

use bitcoin::util::bip32::ExtendedPubKey;
use std::collections::BTreeSet;
use std::str::FromStr;

use gladis::Gladis;
use gtk::prelude::*;
use gtk::{
    glib, Adjustment, Button, Dialog, Entry, Image, Label, ListBox, ListBoxRow, ListStore,
    TextBuffer, ToggleButton, ToolButton, TreeView,
};
use miniscript::Descriptor;
use relm::Relm;
use wallet::hd::{SegmentIndexes, TrackingAccount};

use super::Msg;
use crate::model::{DescriptorClass, Signer};
use crate::view::settings::spending_row;
use crate::view::settings::spending_row::SpendingModel;

// Create the structure that holds the widgets used in the view.
#[derive(Clone, Gladis)]
pub struct Widgets {
    dialog: Dialog,
    save_btn: Button,
    cancel_btn: Button,

    devices_btn: ToolButton,
    addsign_btn: ToolButton,
    removesign_btn: ToolButton,
    signers_tree: TreeView,
    signers_store: ListStore,

    spending_list: ListBox,
    addcond_btn: ToolButton,
    removecond_btn: ToolButton,

    name_fld: Entry,
    fingerprint_fld: Entry,
    xpub_fld: Entry,
    account_adj: Adjustment,
    accfp_fld: Entry,
    derivation_fld: Entry,
    device_lbl: Label,
    device_img: Image,
    device_status_img: Image,
    seed_mine_tgl: ToggleButton,
    seed_extern_tgl: ToggleButton,

    descriptor_buf: TextBuffer,
    descr_legacy_tgl: ToggleButton,
    descr_segwit_tgl: ToggleButton,
    descr_nested_tgl: ToggleButton,
    descr_taproot_tgl: ToggleButton,
    export_core_tgl: ToggleButton,
    export_lnpbp_tgl: ToggleButton,
}

impl Widgets {
    pub fn show_new(&self) {
        // TODO: Update widgets to match new wallet UI
        self.dialog.show();
    }

    pub fn show_view(&self) {
        // TODO: Update widgets to match show wallet settings UI
        self.dialog.show();
    }

    pub fn hide(&self) {
        self.dialog.hide()
    }

    pub(super) fn root(&self) -> Dialog {
        self.dialog.clone()
    }

    pub(super) fn connect(&self, relm: &Relm<super::Component>) {
        connect!(relm, self.save_btn, connect_clicked(_), Msg::Update);
        connect!(relm, self.cancel_btn, connect_clicked(_), Msg::Hide);
        connect!(relm, self.devices_btn, connect_clicked(_), Msg::DevicesList);

        connect!(
            relm,
            self.signers_tree,
            connect_cursor_changed(_),
            Msg::SignerSelect
        );

        connect!(
            relm,
            self.export_core_tgl,
            connect_toggled(_),
            Msg::ExportFormat(false)
        );
        connect!(
            relm,
            self.export_lnpbp_tgl,
            connect_toggled(_),
            Msg::ExportFormat(true)
        );

        connect!(
            relm,
            self.descr_legacy_tgl,
            connect_clicked(_),
            Msg::ToggleClass(DescriptorClass::PreSegwit)
        );
        connect!(
            relm,
            self.descr_segwit_tgl,
            connect_clicked(_),
            Msg::ToggleClass(DescriptorClass::SegwitV0)
        );
        connect!(
            relm,
            self.descr_nested_tgl,
            connect_clicked(_),
            Msg::ToggleClass(DescriptorClass::NestedV0)
        );
        connect!(
            relm,
            self.descr_taproot_tgl,
            connect_clicked(_),
            Msg::ToggleClass(DescriptorClass::TaprootC0)
        );

        connect!(
            relm,
            self.addcond_btn,
            connect_clicked(_),
            Msg::ConditionAdd
        );
        connect!(
            relm,
            self.removecond_btn,
            connect_clicked(_),
            Msg::ConditionRemove
        );
        connect!(
            relm,
            self.spending_list,
            connect_selected_rows_changed(_),
            Msg::ConditionSelect
        );
    }

    pub(super) fn bind_model(&self, relm: &Relm<super::Component>, model: &SpendingModel) {
        let stream = relm.stream().clone();
        self.spending_list.bind_model(Some(model), move |item| {
            spending_row::RowWidgets::init(stream.clone(), item)
        });
    }

    pub fn set_remove_condition(&self, allow: bool) {
        self.removecond_btn.set_sensitive(allow)
    }

    pub fn selected_condition_index(&self) -> Option<i32> {
        self.spending_list
            .selected_row()
            .as_ref()
            .map(ListBoxRow::index)
    }

    pub fn selected_signer_xpub(&self) -> Option<ExtendedPubKey> {
        self.signers_tree
            .selection()
            .selected()
            .map(|(list_model, iter)| list_model.value(&iter, 3))
            .as_ref()
            .map(glib::Value::get::<String>)
            .transpose()
            .expect("unable to get xpub value from tree column")
            .as_deref()
            .map(ExtendedPubKey::from_str)
            .transpose()
            .expect("invalid signer xpub")
    }

    pub fn update_signer_details(&self, details: Option<(&Signer, TrackingAccount)>) {
        if let Some((signer, ref derivation)) = details {
            self.name_fld.set_text(&signer.name);
            self.fingerprint_fld
                .set_text(&signer.fingerprint.to_string());
            self.xpub_fld.set_text(&signer.xpub.to_string());
            self.account_adj
                .set_value(signer.account.first_index() as f64);
            self.accfp_fld
                .set_text(&signer.xpub.fingerprint().to_string());
            self.derivation_fld.set_text(&derivation.to_string());
        }
        if let Some((device, model)) =
            details.and_then(|(s, _)| s.device.as_ref().map(|d| (d, &s.name)))
        {
            self.device_img.set_visible(true);
            self.device_status_img.set_visible(true);
            self.device_lbl.set_text(&format!("{} ({})", device, model));
        } else {
            self.device_img.set_visible(false);
            self.device_status_img.set_visible(false);
            self.device_lbl.set_text("Unknown");
        }
    }

    pub fn update_signers(&mut self, signers: &BTreeSet<Signer>) {
        let store = &mut self.signers_store;
        store.clear();
        for signer in signers {
            store.insert_with_values(
                None,
                &[
                    (0, &signer.name),
                    (1, &signer.fingerprint.to_string()),
                    (2, &signer.account.to_string()),
                    (3, &signer.xpub.to_string()),
                    (4, &signer.device.clone().unwrap_or_default()),
                ],
            );
        }
    }

    pub fn update_descriptor(
        &mut self,
        descriptor: Option<&Descriptor<TrackingAccount>>,
        format: bool,
    ) {
        let text = match (descriptor, format) {
            (Some(descriptor), false) => format!("{:#}", descriptor),
            (Some(descriptor), true) => format!("{}", descriptor),
            (None, _) => s!(""),
        };
        self.descriptor_buf.set_text(&text);
    }

    fn descr_class_toggle(&self, class: DescriptorClass) -> &ToggleButton {
        match class {
            DescriptorClass::PreSegwit => &self.descr_legacy_tgl,
            DescriptorClass::SegwitV0 => &self.descr_segwit_tgl,
            DescriptorClass::NestedV0 => &self.descr_nested_tgl,
            DescriptorClass::TaprootC0 => &self.descr_taproot_tgl,
        }
    }

    pub fn should_update_descr_class(&mut self, class: DescriptorClass) -> bool {
        self.descr_class_toggle(class).is_active()
    }

    pub fn update_descr_class(&mut self, class: DescriptorClass) {
        self.descr_legacy_tgl
            .set_active(class == DescriptorClass::PreSegwit);
        self.descr_segwit_tgl
            .set_active(class == DescriptorClass::SegwitV0);
        self.descr_nested_tgl
            .set_active(class == DescriptorClass::NestedV0);
        self.descr_taproot_tgl
            .set_active(class == DescriptorClass::TaprootC0);
    }
}