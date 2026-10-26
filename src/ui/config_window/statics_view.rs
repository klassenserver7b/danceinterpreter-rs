//! Statics panel: dance labels and announcements with favorite star, color
//! swatch + picker, inline name editing, merge-on-submit and show-now.

use crate::dataloading::dataprovider::{ItemChange, ItemSource};
use crate::dataloading::staticinfo::StaticInfo;
use crate::ui::config_window::ConfigWindow;
use crate::ui::theme::RowLook;
use crate::ui::config_window::search::data_matches_search_query;
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{
    buttons, heading, material_symbol, material_symbol_colored, material_symbol_sized,
};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, DialogMessage, Message, StaticsMessage};
use iced::widget::{
    Column, Space, button, column as col, container, row, scrollable, text, text_input,
};
use iced::{Alignment, Color, Element, Length, Theme};

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
            heading("No statics yet.", 18.0, iced::font::Weight::Semibold).style(|t| {
                iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).text),
                }
            }),
            text("Statics appear automatically from playlist dances. Add one below.")
                .size(sz::SMALL)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).muted),
                }),
            buttons::primary_small(
                "Add static",
                Message::OpenDialog(crate::ui::config_window::DialogState::AddStatic),
            ),
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

    let toolbar = container(
        row![
            row![
                material_symbol_sized("grid_view", false, 15.0).style(|t: &Theme| {
                    iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).muted),
                    }
                }),
                heading("Dances & announcements", sz::PANEL_TITLE, iced::font::Weight::Semibold)
                    .style(|t| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).text),
                    }),
            ]
            .spacing(9)
            .align_y(Alignment::Center),
            container(
                buttons::primary_small(
                    "Add static",
                    Message::OpenDialog(crate::ui::config_window::DialogState::AddStatic),
                )
            )
            .width(Length::Fill)
            .align_x(Alignment::End),
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center),
    )
    .padding([14, 17])
    .width(Length::Fill);

    let panel = col![toolbar, list].width(Length::Fill).height(Length::Fill);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::card)
        .clip(true)
        .into()
}

const COL_DRAG: f32 = 16.0;
const COL_FAV: f32 = 26.0;
const COL_COLOR: f32 = 26.0;
const COL_CAPTION: f32 = 70.0;
const ROW_HEIGHT: f32 = 62.0;

#[allow(clippy::too_many_arguments)]
fn build_static_row<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
    static_info: &'a StaticInfo,
    name: String,
    idx: usize,
    is_match: bool,
) -> Element<'a, Message> {
    let is_current = *dance_interpreter.data_provider.get_current_source()
        == ItemSource::Static(name.clone());

    // Favorite star.
    let star: Element<'a, Message> = if static_info.is_favorite {
        button(
            material_symbol_colored("star", true, theme::Tokens::of(&config_window.theme).star),
        )
        .padding(4)
        .style(theme::btn_ghost)
        .on_press(Message::Statics(StaticsMessage::ToggleFavorite(name.clone())))
        .into()
    } else {
        button(material_symbol("star", false))
            .padding(4)
            .style(theme::btn_ghost)
            .on_press(Message::Statics(StaticsMessage::ToggleFavorite(name.clone())))
            .into()
    };

    // Color swatch / color picker.
    let swatch_color = static_info
        .color
        .unwrap_or_else(|| theme::Tokens::dark().muted);
    let color_picker_open = config_window.color_picker_open.as_ref() == Some(&name);
    let swatch = crate::ui::widgets::color_swatch::color_swatch(
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
        .on_input(move |v| Message::Statics(StaticsMessage::UpdateName(edit_name.clone(), v)))
        .on_submit(Message::Statics(StaticsMessage::SubmitName(name.clone())))
        .style(theme::input_plain)
        .padding(0);

    let show_now = button(
        row![
            material_symbol_sized("play_arrow", false, 13.0),
            text("Show now").size(sz::TINY),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([5, 10])
    .style(theme::btn_secondary)
    .on_press(Message::ItemChanged(ItemChange::StaticAbsolute(name.clone())));

    let delete = with_tooltip(
        buttons::ghost_icon(
            "delete",
            false,
            14.0,
            Message::Dialog(DialogMessage::RequestDelete(ItemSource::Static(name.clone()))),
        ),
        "Delete static",
    );

    let content = row![
        drag_handle(),
        container(star).width(Length::Fixed(COL_FAV)),
        container(color_element).width(Length::Fixed(COL_COLOR)),
        container(name_input).width(Length::Fill).clip(true),
        container(
            text("Text label")
                .size(sz::TINY)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).faint),
                })
        )
        .width(Length::Fixed(COL_CAPTION))
        .align_x(Alignment::End),
        show_now,
        delete,
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .padding(crate::ui::theme::pad4(0.0, 19.0, 0.0, 11.0))
    .width(Length::Fill);

    let bar_width = if is_current { 3.0 } else { 0.0 };
    let bar = container(Space::new())
        .width(Length::Fixed(bar_width))
        .height(Length::Fill)
        .style(move |t: &Theme| {
            container::Style::default().background(if is_current {
                static_info
                    .color
                    .unwrap_or_else(|| theme::Tokens::of(t).accent_bar)
            } else {
                Color::TRANSPARENT
            })
        });

    let look = RowLook {
        zebra: idx % 2 == 1,
        current: is_current,
        matched: is_match && !is_current,
        first: false,
    };

    container(row![bar, content])
        .height(Length::Fixed(ROW_HEIGHT))
        .width(Length::Fill)
        .style(move |t: &Theme| theme::row(t, look))
        .into()
}

fn drag_handle() -> Element<'static, Message> {
    container(
        material_symbol_sized("drag_indicator", false, 15.0).style(|t: &Theme| {
            iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).border),
            }
        }),
    )
    .width(Length::Fixed(COL_DRAG))
    .align_x(Alignment::Center)
    .into()
}
