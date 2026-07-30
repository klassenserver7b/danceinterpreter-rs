use crate::dataloading::dataprovider::ItemChange;
use crate::dataloading::dataprovider::ItemSource;
use crate::dataloading::staticinfo::StaticInfo;
use crate::ui::config_window::ConfigWindow;
use crate::ui::config_window::item_row::build_item_row_container_styled;
use crate::ui::config_window::search_bar::data_matches_search_query;
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::ui::widgets::buttons::{
    material_symbol_message_button, material_symbol_message_button_colored,
};
use crate::ui::widgets::material_symbol;
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{
    Column, Row, Scrollable, Space, button, column as col, container, row, scrollable, text,
};
use iced::{Alignment, Color, Element, Length};

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Column<'a, Message> {
    let trow = container(
        row![
            text("Fav").width(Length::Fixed(42.0)),
            text!("Name").width(Length::Fill),
            Space::new().width(Length::Fill).height(Length::Shrink),
            button(row![material_symbol("add", false), text("Add Static")].spacing(5))
                .on_press(Message::AddBlankStatic)
                .padding([5, 10])
                .style(button::primary),
        ]
        .align_y(Vertical::Bottom)
        .spacing(5),
    )
    .padding([4, 6])
    .width(Length::Fill);

    let mut list_column = col!().spacing(5);

    for (i, (name, static_info)) in dance_interpreter.data_provider.statics().iter().enumerate() {
        let is_match = data_matches_search_query(&config_window.search_query, static_info.into());
        let static_row = build_static_row(
            static_info,
            name.clone(),
            config_window.theme.extended_palette().primary.weak.color,
            config_window.color_picker_open.as_ref() == Some(name),
        );
        let accent_color = static_info.color;
        let row_container = build_item_row_container_styled(static_row, is_match, i, accent_color);
        list_column = list_column.push(row_container);
    }

    let statics_scrollable: Scrollable<'_, Message> = scrollable(list_column)
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(5);

    col!(trow, statics_scrollable).spacing(5)
}

pub fn build_static_row<'a>(
    static_info: &StaticInfo,
    name: String,
    color: impl Into<Color>,
    color_picker_open: bool,
) -> Row<'a, Message> {
    let swatch_color = static_info.color.unwrap_or(Color::TRANSPARENT);
    let mut name_clone = name.clone();
    let mut name_clone1 = name.clone();

    let color_swatch = crate::ui::widgets::color_swatch::color_swatch(
        swatch_color,
        Message::ToggleStaticColorPicker(name.clone()),
    );

    let color_element: Element<'a, Message> = if color_picker_open {
        iced_aw::ColorPicker::new(
            true,
            swatch_color,
            color_swatch,
            Message::ToggleStaticColorPicker(name.clone()),
            move |c| Message::UpdateStaticColor(name_clone.clone(), c),
        )
        .on_color_change(move |c| Message::PreviewStaticColor(name_clone1.clone(), c))
        .into()
    } else {
        color_swatch
    };

    name_clone = name.clone();
    name_clone1 = name.clone();

    row![
        if static_info.is_favorite {
            material_symbol_message_button_colored(
                "star",
                static_info.is_favorite,
                Message::ToggleStaticFavorite(name.clone()),
                color,
            )
        } else {
            material_symbol_message_button(
                "star",
                static_info.is_favorite,
                Message::ToggleStaticFavorite(name.clone()),
            )
        },
        Space::new().width(5),
        color_element,
        DynamicTextInput::<'_, Message>::new("Static Name", &static_info.name)
            .width(Length::Fill)
            .on_change(move |v| Message::UpdateStaticName(name_clone.clone(), v))
            .on_submit(Message::SubmitStaticName(name_clone1.clone())),
        row![
            Space::new().width(Length::Fill).height(Length::Shrink),
            with_tooltip(
                material_symbol_message_button(
                    "smart_display",
                    false,
                    Message::ItemChanged(ItemChange::StaticAbsolute(name.clone()))
                ),
                "Show now"
            ),
            with_tooltip(
                material_symbol_message_button(
                    "delete",
                    false,
                    Message::RequestDelete(ItemSource::Static(name.clone()))
                ),
                "Delete static"
            ),
        ]
        .spacing(5)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(10)
}
