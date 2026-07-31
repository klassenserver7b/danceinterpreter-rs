use crate::dataloading::displayable_data::DisplayableData;
use crate::ui::widgets::buttons::material_symbol_message_button;
use crate::ui::widgets::material_symbol;
use crate::{Message, SearchMessage};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2 as Matcher;
use iced::alignment::Vertical;
use iced::widget::{Row, TextInput, row};
use iced::{Alignment, Length};

pub fn build_search_bar<'a>(search_query: &str) -> Row<'a, Message> {
    row![
        material_symbol("search", false)
            .width(Length::Fixed(24.0))
            .align_y(Vertical::Center),
        TextInput::new("Search...", search_query)
            .on_input(|v| Message::Search(SearchMessage::Changed(v)))
            .on_submit(Message::Noop)
            .width(Length::Fill)
            .padding([4, 8]),
        material_symbol_message_button("backspace", false, Message::Search(SearchMessage::Clear)),
        material_symbol_message_button("close", false, Message::Search(SearchMessage::Toggle)),
    ]
    .spacing(5)
    .padding([5, 5])
    .align_y(Alignment::Center)
}

pub fn data_matches_search_query(search_query: &str, data: DisplayableData) -> bool {
    let matcher = Matcher::default();

    !search_query.is_empty()
        && matcher
            .fuzzy_match(
                format!(
                    "{} {} {}",
                    data.headline.to_lowercase(),
                    data.subline_upper.to_lowercase(),
                    data.subline_lower.to_lowercase()
                )
                .trim(),
                &search_query.to_lowercase(),
            )
            .is_some()
}
