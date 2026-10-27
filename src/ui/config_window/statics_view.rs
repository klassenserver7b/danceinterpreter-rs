//! Statics panel: dance labels and announcements with favorite star, color
//! swatch + picker, inline name editing, merge-on-submit and show-now.

use crate::dataloading::dataprovider::{ItemChange, ItemSource};
use crate::dataloading::staticinfo::StaticInfo;
use crate::ui::config_window::search::data_matches_search_query;
use crate::ui::config_window::{ConfigWindow, DialogState};
use crate::ui::theme::RowLook;
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{
    buttons, card_panel, color_swatch, delete_button, heading, icon_button_tooltip,
    material_symbol, material_symbol_sized, panel_header, row_shell,
};
use crate::{DanceInterpreter, Message, StaticsMessage};
use iced::widget::{
    Column, Space, button, column as col, container, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length};

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let mut rows: Column<'_, Message> = col![];

    for (i, (name, static_info)) in dance_interpreter.data_provider.statics().iter().enumerate() {
        let is_match = data_matches_search_query(&config_window.search_query, static_info.into());
        rows = rows.push(build_static_row(
            config_window,
            dance_interpreter,
            static_info,
            name.clone(),
            i,
            is_match,
        ));
    }

    let list: Element<'_, Message> = if dance_interpreter.data_provider.statics().is_empty() {
        col![
            Space::new().height(Length::Fill),
            heading("No statics yet.", sz::H2, iced::font::Weight::Semibold)
                .style(theme::text_style(|tk| tk.text)),
            text("Statics appear automatically from playlist dances. Add one below.")
                .size(sz::SMALL)
                .style(theme::text_style(|tk| tk.muted)),
            buttons::primary_small("Add static", Message::OpenDialog(DialogState::AddStatic),),
            Space::new().height(Length::Fill),
        ]
        .spacing(10)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        scrollable(rows)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::scrollable_style)
            .into()
    };

    let tools = row![
        icon_button_tooltip(
            "refresh",
            sz::ICON,
            Message::Statics(StaticsMessage::Reload),
            "Reload statics",
        ),
        buttons::primary_small("Add static", Message::OpenDialog(DialogState::AddStatic),),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let toolbar = panel_header("grid_view", "Dances & announcements", "", tools.into());

    let panel = col![toolbar, list].width(Length::Fill).height(Length::Fill);

    card_panel(panel).into()
}

const COL_FAV: f32 = 26.0;
const COL_COLOR: f32 = 26.0;
const COL_CAPTION: f32 = 70.0;

fn build_static_row<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
    static_info: &'a StaticInfo,
    name: String,
    idx: usize,
    is_match: bool,
) -> Element<'a, Message> {
    let is_current = matches!(
        dance_interpreter.data_provider.get_current_source(),
        ItemSource::Static(current) if *current == name
    );

    let star_icon = if static_info.is_favorite {
        material_symbol("star", true).style(theme::text_style(|tk| tk.star))
    } else {
        material_symbol("star", false)
    };
    let star = button(star_icon)
        .padding(4)
        .style(theme::btn_ghost)
        .on_press(Message::Statics(StaticsMessage::ToggleFavorite(
            name.clone(),
        )));

    // Color swatch / color picker.
    let swatch_color = static_info
        .color
        .unwrap_or_else(|| theme::Tokens::dark().muted);
    let color_picker_open = config_window.color_picker_open.as_ref() == Some(&name);
    let swatch = color_swatch::color_swatch(
        swatch_color,
        Message::Statics(StaticsMessage::ToggleColorPicker(name.clone())),
    );

    let color_element: Element<'a, Message> = if color_picker_open {
        let close_name = name.clone();
        let submit_name = name.clone();
        let preview_name = name.clone();
        iced_aw::ColorPicker::new(
            true,
            swatch_color,
            swatch,
            Message::Statics(StaticsMessage::ToggleColorPicker(close_name)),
            move |c| Message::Statics(StaticsMessage::UpdateColor(submit_name.clone(), c)),
        )
        .on_color_change(move |c| {
            Message::Statics(StaticsMessage::PreviewColor(preview_name.clone(), c))
        })
        .into()
    } else {
        swatch
    };

    // Name (inline editable; submitting an existing name opens the merge dialog).
    let edit_name = name.clone();
    let name_input = text_input("Static name", &static_info.name)
        .size(sz::ROW)
        .on_input(move |v| Message::Statics(StaticsMessage::UpdateName(edit_name.clone(), v)))
        .on_submit(Message::Statics(StaticsMessage::SubmitName(name.clone())))
        .style(theme::input_plain)
        .padding(0);

    let show_now = button(
        row![
            material_symbol_sized("play_arrow", false, sz::ICON_XXS),
            text("Show now").size(sz::SMALL),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([5, 10])
    .style(theme::btn_secondary)
    .on_press(Message::ItemChanged(ItemChange::StaticAbsolute(
        name.clone(),
    )));

    let delete = delete_button(ItemSource::Static(name.clone()), "Delete static");

    // How many playlist songs use this dance.
    let usage = dance_interpreter
        .data_provider
        .playlist()
        .iter()
        .filter(|item| item.song.dance == *name)
        .count();
    let usage_label = match usage {
        0 => "Unused".to_string(),
        1 => "1 song".to_string(),
        n => format!("{n} songs"),
    };

    let content = row![
        container(star).width(Length::Fixed(COL_FAV)),
        container(color_element).width(Length::Fixed(COL_COLOR)),
        container(name_input).width(Length::Fill).clip(true),
        container(
            text(usage_label)
                .size(sz::TINY)
                .style(theme::text_style(|tk| tk.faint))
        )
        .width(Length::Fixed(COL_CAPTION))
        .align_x(Alignment::End),
        show_now,
        delete,
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .padding(pad4(0.0, 19.0, 0.0, 14.0))
    .width(Length::Fill);

    let look = RowLook::table(idx, is_current, is_match);

    // The current static's bar takes its dance color when it has one.
    let static_color = static_info.color;
    row_shell(content, look, move |t| {
        static_color.unwrap_or_else(|| theme::Tokens::of(t).accent_bar)
    })
}
