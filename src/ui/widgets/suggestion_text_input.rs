//! A text input with a searchable suggestion dropdown, vendored from
//! iced's `combo_box` (`iced_widget-0.14`) and adapted: options come from a
//! plain list instead of a `State<T>`-managed cursor, the input always shows
//! the current value, and Enter submits the typed text (not just a picked
//! option). Used for the Traktor server address field.
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, renderer, text, widget};
use iced::keyboard;
use iced::keyboard::key;
use iced::mouse;
use iced::overlay;
use iced::overlay::menu;
use iced::time::Instant;
use iced::{Element, Event, Length, Padding, Pixels, Rectangle, Size, Theme, Vector};

use iced::widget::{TextInput, text_input};
use std::cell::RefCell;
use std::fmt::Display;

/// A widget for searching and selecting a single value from a list of options.
///
/// Unlike the upstream `combo_box`, the embedded text input always shows the
/// given `selection` (or the last typed text) rather than a placeholder, and
/// the option list is supplied per call site instead of living in `State`.
pub struct SuggestionTextInput<'a, T, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    state: &'a State<T>,
    text_input: TextInput<'a, TextInputEvent, Theme, Renderer>,
    selection: text_input::Value,
    on_selected: Box<dyn Fn(T) -> Message>,
    on_option_hovered: Option<Box<dyn Fn(T) -> Message>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message>>,
    padding: Padding,
    size: Option<f32>,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
}

impl<'a, T, Message, Theme, Renderer> SuggestionTextInput<'a, T, Message, Theme, Renderer>
where
    T: Display + Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new [`SuggestionTextInput`] with the given list of options, a placeholder,
    /// the current selected value, and the message to produce when an option is
    /// selected.
    pub fn new(
        state: &'a State<T>,
        placeholder: &str,
        selection: Option<&T>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        let text_input = TextInput::new(placeholder, &state.value())
            .on_input(TextInputEvent::TextChanged)
            .class(Theme::default_input());

        let selection = selection.map(T::to_string).unwrap_or_default();

        Self {
            state,
            text_input,
            selection: text_input::Value::new(&selection),
            on_selected: Box::new(on_selected),
            on_option_hovered: None,
            on_input: None,
            on_open: None,
            on_close: None,
            padding: text_input::DEFAULT_PADDING,
            size: None,
            menu_class: <Theme as Catalog>::default_menu(),
        }
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`] of the [`SuggestionTextInput`].
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'static) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that will be produced when an option of the
    /// [`SuggestionTextInput`] is hovered using the arrow keys.
    pub fn on_option_hovered(mut self, on_option_hovered: impl Fn(T) -> Message + 'static) -> Self {
        self.on_option_hovered = Some(Box::new(on_option_hovered));
        self
    }

    /// Sets the message that will be produced when the  [`SuggestionTextInput`] is
    /// opened.
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    /// Sets the message that will be produced when the outside area
    /// of the [`SuggestionTextInput`] is pressed.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the [`Padding`] of the [`SuggestionTextInput`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self.text_input = self.text_input.padding(self.padding);
        self
    }

    /// Sets the text size of the [`SuggestionTextInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        let size = size.into();

        self.text_input = self.text_input.size(size);
        self.size = Some(size.0);

        self
    }

    /// Sets the style of the input of the [`SuggestionTextInput`].
    #[must_use]
    pub fn input_style(
        mut self,
        style: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self
    where
        <Theme as text_input::Catalog>::Class<'a>: From<text_input::StyleFn<'a, Theme>>,
    {
        self.text_input = self.text_input.style(style);
        self
    }

    /// Sets the style of the menu of the [`SuggestionTextInput`].
    #[must_use]
    pub fn menu_style(mut self, style: impl Fn(&Theme) -> menu::Style + 'a) -> Self
    where
        <Theme as menu::Catalog>::Class<'a>: From<menu::StyleFn<'a, Theme>>,
    {
        self.menu_class = (Box::new(style) as menu::StyleFn<'a, Theme>).into();
        self
    }
}

/// The local state of a [`SuggestionTextInput`].
#[derive(Debug, Clone)]
pub struct State<T> {
    options: Vec<T>,
    inner: RefCell<Inner<T>>,
}

#[derive(Debug, Clone)]
struct Inner<T> {
    value: String,
    option_matchers: Vec<String>,
    filtered_options: Filtered<T>,
}

impl<T> Inner<T>
where
    T: Clone,
{
    /// Resets the typed value and restores the full (unfiltered) option list.
    fn reset(&mut self, options: Vec<T>) {
        self.value = String::new();
        self.filtered_options.update(options);
    }
}

#[derive(Debug, Clone)]
struct Filtered<T> {
    options: Vec<T>,
    updated: Instant,
}

impl<T> State<T>
where
    T: Display + Clone,
{
    /// Creates a new [`State`] for a [`SuggestionTextInput`] with the given list of options.
    pub fn new(options: Vec<T>) -> Self {
        Self::with_selection(options, None)
    }

    /// Creates a new [`State`] for a [`SuggestionTextInput`] with the given list of options
    /// and selected value.
    pub fn with_selection(options: Vec<T>, selection: Option<&T>) -> Self {
        let value = selection.map(T::to_string).unwrap_or_default();

        // Pre-build "matcher" strings ahead of time so that search is fast
        let option_matchers = build_matchers(&options);

        let filtered_options = Filtered::new(
            search(&options, &option_matchers, &value)
                .cloned()
                .collect(),
        );

        Self {
            options,
            inner: RefCell::new(Inner {
                value,
                option_matchers,
                filtered_options,
            }),
        }
    }

    fn value(&self) -> String {
        let inner = self.inner.borrow();

        inner.value.clone()
    }

    fn with_inner<O>(&self, f: impl FnOnce(&Inner<T>) -> O) -> O {
        let inner = self.inner.borrow();

        f(&inner)
    }

    fn with_inner_mut(&self, f: impl FnOnce(&mut Inner<T>)) {
        let mut inner = self.inner.borrow_mut();

        f(&mut inner);
    }

    fn sync_filtered_options(&self, options: &mut Filtered<T>) {
        let inner = self.inner.borrow();

        inner.filtered_options.sync(options);
    }
}

impl<T> Default for State<T>
where
    T: Display + Clone,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T> Filtered<T>
where
    T: Clone,
{
    fn new(options: Vec<T>) -> Self {
        Self {
            options,
            updated: Instant::now(),
        }
    }

    fn update(&mut self, options: Vec<T>) {
        self.options = options;
        self.updated = Instant::now();
    }

    fn sync(&self, other: &mut Filtered<T>) {
        if other.updated != self.updated {
            *other = self.clone();
        }
    }
}

struct Menu<T> {
    menu: menu::State,
    hovered_option: Option<usize>,
    new_selection: Option<T>,
    filtered_options: Filtered<T>,
}

#[derive(Debug, Clone)]
enum TextInputEvent {
    TextChanged(String),
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SuggestionTextInput<'_, T, Message, Theme, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Widget::<TextInputEvent, Theme, Renderer>::size(&self.text_input)
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let is_focused = text_input_focused::<Renderer>(tree);

        self.text_input.layout(
            &mut tree.children[0],
            renderer,
            limits,
            (!is_focused).then_some(&self.selection),
        )
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let is_focused = text_input_focused::<Renderer>(tree);

        let selection = if is_focused || self.selection.is_empty() {
            None
        } else {
            Some(&self.selection)
        };

        self.text_input.draw(
            &tree.children[0],
            renderer,
            theme,
            layout,
            cursor,
            selection,
            viewport,
        );
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Menu<T>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Menu::<T> {
            menu: menu::State::new(),
            filtered_options: Filtered::new(Vec::new()),
            hovered_option: Some(0),
            new_selection: None,
        })
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.text_input as &dyn Widget<_, _, _>)]
    }

    fn diff(&self, _tree: &mut widget::Tree) {
        // do nothing so the children don't get cleared
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let started_focused = text_input_focused::<Renderer>(tree);

        // Create a new list of local messages
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        // Provide it to the widget
        self.text_input.update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );

        if local_shell.is_event_captured() {
            shell.capture_event();
        }

        shell.request_redraw_at(local_shell.redraw_request());
        shell.request_input_method(local_shell.input_method());

        // Then finally react to them here
        self.handle_text_changed(local_messages, tree, shell);

        self.handle_keyboard(started_focused, tree, event, shell);

        // If the overlay menu has selected something
        self.commit_pending_selection(tree, layout, renderer, clipboard, shell, viewport);

        let is_focused = text_input_focused::<Renderer>(tree);

        if started_focused != is_focused {
            // Focus changed, invalidate widget tree to force a fresh `view`
            shell.invalidate_widgets();

            if is_focused {
                if let Some(on_open) = self.on_open.take() {
                    shell.publish(on_open);
                }
            } else if let Some(on_close) = self.on_close.take() {
                shell.publish(on_close);
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.text_input
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if text_input_focused::<Renderer>(tree) {
            let Menu {
                menu,
                filtered_options,
                hovered_option,
                ..
            } = tree.state.downcast_mut::<Menu<T>>();

            self.state.sync_filtered_options(filtered_options);

            if filtered_options.options.is_empty() {
                None
            } else {
                let bounds = layout.bounds();

                let mut menu = menu::Menu::new(
                    menu,
                    &filtered_options.options,
                    hovered_option,
                    |selection| {
                        self.state.with_inner_mut(|state| {
                            state.reset(self.state.options.clone());
                        });

                        tree.children[0]
                            .state
                            .downcast_mut::<text_input::State<Renderer::Paragraph>>()
                            .unfocus();

                        (self.on_selected)(selection)
                    },
                    self.on_option_hovered.as_deref(),
                    &self.menu_class,
                )
                .width(bounds.width)
                .padding(self.padding);

                if let Some(size) = self.size {
                    menu = menu.text_size(size);
                }

                Some(menu.overlay(
                    layout.position() + translation,
                    *viewport,
                    bounds.height,
                    Length::Shrink,
                ))
            }
        } else {
            None
        }
    }
}

impl<T, Message, Theme, Renderer> SuggestionTextInput<'_, T, Message, Theme, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Publishes [`Self::on_input`] for every text change the embedded input
    /// produced and re-filters the option list for the new query.
    fn handle_text_changed(
        &mut self,
        local_messages: Vec<TextInputEvent>,
        tree: &mut widget::Tree,
        shell: &mut Shell<'_, Message>,
    ) {
        for TextInputEvent::TextChanged(new_value) in local_messages {
            if let Some(on_input) = &self.on_input {
                shell.publish(on_input(new_value.clone()));
            }

            // Couple the filtered options with the `SuggestionTextInput`
            // value and only recompute them when the value changes,
            // instead of doing it in every `view` call
            let menu = tree.state.downcast_mut::<Menu<T>>();
            self.state.with_inner_mut(|state| {
                menu.hovered_option = Some(0);
                state.value = new_value;

                state.filtered_options.update(
                    search(&self.state.options, &state.option_matchers, &state.value)
                        .cloned()
                        .collect(),
                );
            });
            shell.invalidate_layout();
            shell.request_redraw();
        }
    }

    /// Reacts to keyboard events while the input is focused: on first focus
    /// publishes [`Self::on_option_hovered`], the arrow / tab keys cycle the
    /// hover highlight and Enter marks the hovered option for selection.
    fn handle_keyboard(
        &mut self,
        started_focused: bool,
        tree: &mut widget::Tree,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) {
        if !text_input_focused::<Renderer>(tree) {
            return;
        }

        let menu = tree.state.downcast_mut::<Menu<T>>();

        self.state.with_inner(|state| {
            if !started_focused && let Some(on_option_hovered) = &mut self.on_option_hovered {
                let hovered_option = menu.hovered_option.unwrap_or(0);

                if let Some(option) = state.filtered_options.options.get(hovered_option) {
                    shell.publish(on_option_hovered(option.clone()));
                }
            }

            let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named_key),
                modifiers,
                ..
            }) = event
            else {
                return;
            };

            let shift_modifier = modifiers.shift();

            // Arrow keys and Tab cycle the hover highlight through the
            // option list (wrapping around) and publish the newly hovered
            // option.
            let cycle = match (named_key, shift_modifier) {
                (key::Named::ArrowUp, _) | (key::Named::Tab, true) => Some(-1),
                (key::Named::ArrowDown, _) | (key::Named::Tab, false) if !modifiers.shift() => {
                    Some(1)
                }
                _ => None,
            };

            if let Some(delta) = cycle {
                cycle_hovered(menu, state.filtered_options.options.len(), delta);

                if let Some(on_option_hovered) = &mut self.on_option_hovered
                    && let Some(option) = menu
                        .hovered_option
                        .and_then(|index| state.filtered_options.options.get(index))
                {
                    // Notify the selection
                    shell.publish(on_option_hovered(option.clone()));
                }

                shell.capture_event();
                shell.request_redraw();
            }

            if named_key == &key::Named::Enter {
                if let Some(index) = &menu.hovered_option
                    && let Some(option) = state.filtered_options.options.get(*index)
                {
                    menu.new_selection = Some(option.clone());
                }

                shell.capture_event();
                shell.request_redraw();
            }
        });
    }

    /// Commits a pending menu selection (Enter or click): resets the query,
    /// restores the full option list, publishes [`Self::on_selected`] and
    /// unfocuses the embedded input.
    fn commit_pending_selection(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let menu = tree.state.downcast_mut::<Menu<T>>();

        self.state.with_inner_mut(|state| {
            let Some(selection) = menu.new_selection.take() else {
                return;
            };

            // Clear the value and reset the options and menu
            state.reset(self.state.options.clone());
            menu.menu = menu::State::default();

            // Notify the selection
            shell.publish((self.on_selected)(selection));

            // Unfocus the input
            let mut local_messages = Vec::new();
            let mut local_shell = Shell::new(&mut local_messages);
            self.text_input.update(
                &mut tree.children[0],
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Unavailable,
                renderer,
                clipboard,
                &mut local_shell,
                viewport,
            );
            shell.request_input_method(local_shell.input_method());
        });
    }
}

impl<'a, T, Message, Theme, Renderer> From<SuggestionTextInput<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(suggestion_text_input: SuggestionTextInput<'a, T, Message, Theme, Renderer>) -> Self {
        Self::new(suggestion_text_input)
    }
}

/// The theme catalog of a [`SuggestionTextInput`].
pub trait Catalog: text_input::Catalog + menu::Catalog {
    /// The default class for the text input of the [`SuggestionTextInput`].
    fn default_input<'a>() -> <Self as text_input::Catalog>::Class<'a> {
        <Self as text_input::Catalog>::default()
    }

    /// The default class for the menu of the [`SuggestionTextInput`].
    fn default_menu<'a>() -> <Self as menu::Catalog>::Class<'a> {
        <Self as menu::Catalog>::default()
    }
}

impl Catalog for Theme {}

fn search<'a, T, A>(
    options: impl IntoIterator<Item = T> + 'a,
    option_matchers: impl IntoIterator<Item = &'a A> + 'a,
    query: &'a str,
) -> impl Iterator<Item = T> + 'a
where
    A: AsRef<str> + 'a,
{
    let query: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(String::from)
        .collect();

    options
        .into_iter()
        .zip(option_matchers)
        // Make sure each part of the query is found in the option
        .filter_map(move |(option, matcher)| {
            if query.iter().all(|part| matcher.as_ref().contains(part)) {
                Some(option)
            } else {
                None
            }
        })
}

fn build_matchers<'a, T>(options: impl IntoIterator<Item = T> + 'a) -> Vec<String>
where
    T: Display + 'a,
{
    options.into_iter().map(build_matcher).collect()
}

fn build_matcher<T>(option: T) -> String
where
    T: Display,
{
    let mut matcher = option.to_string();
    matcher.retain(|c| c.is_ascii_alphanumeric());
    matcher.to_lowercase()
}

/// Whether the embedded text input of the widget in `tree` is focused.
fn text_input_focused<Renderer: text::Renderer>(tree: &widget::Tree) -> bool {
    tree.children[0]
        .state
        .downcast_ref::<text_input::State<Renderer::Paragraph>>()
        .is_focused()
}

/// Moves the menu's hovered option by `delta` (-1 or 1), wrapping around the
/// option list; an unset hover starts at the first option.
fn cycle_hovered<T>(menu: &mut Menu<T>, count: usize, delta: isize) {
    let next = match menu.hovered_option {
        Some(index) if count > 0 => {
            let step = delta.rem_euclid(count as isize) as usize;
            (index + step) % count
        }
        _ => 0,
    };

    menu.hovered_option = Some(next);
}
