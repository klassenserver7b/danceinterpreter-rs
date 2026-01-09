use iced::advanced::graphics::core::{touch, Element};
use iced::advanced::text::Wrapping;
use iced::advanced::widget::{tree, Operation, Tree};
use iced::advanced::{layout, mouse, overlay, renderer, text, Clipboard, Layout, Shell, Widget};
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::widget::text::{LineHeight, Shaping};
use iced::widget::{text_input, Text, TextInput};
use iced::{alignment, keyboard, Color, Event, Length, Pixels, Rectangle, Size, Vector};

#[allow(missing_debug_implementations)]
pub struct DynamicTextInput<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: 'a + text::Renderer,
    Message: 'a + Clone,
    Theme: 'a + text_input::Catalog + iced::widget::text::Catalog,
{
    content_label: Text<'a, Theme, Renderer>,
    content_input: TextInput<'a, Message, Theme, Renderer>,

    width: Length,
    interaction: Option<mouse::Interaction>,

    on_enter: Option<Message>,
    on_submit: Option<Message>,
}

impl<'a, Message, Theme, Renderer> DynamicTextInput<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
    Message: Clone,
    Theme: text_input::Catalog + iced::widget::text::Catalog,
{
    pub fn interaction(mut self, interaction: mouse::Interaction) -> Self {
        self.interaction = Some(interaction);
        self
    }

    fn get_widget(&self, is_edit_mode: bool) -> &dyn Widget<Message, Theme, Renderer> {
        if is_edit_mode {
            &self.content_input
        } else {
            &self.content_label
        }
    }

    fn get_widget_mut(&mut self, is_edit_mode: bool) -> &mut dyn Widget<Message, Theme, Renderer> {
        if is_edit_mode {
            &mut self.content_input
        } else {
            &mut self.content_label
        }
    }
}

#[derive(Default)]
struct State {
    is_pressed: bool,
    is_edit_mode: bool,

    previous_click: Option<mouse::Click>,
}

impl State {
    fn get_child_index(&self) -> usize {
        if self.is_edit_mode { 1 } else { 0 }
    }
}

fn get_placeholder_color<Theme: text_input::Catalog>(theme: &Theme) -> Color {
    let class = Theme::default();
    let style = theme.style(&class, text_input::Status::Active);

    style.placeholder
}

impl<'a, Message, Theme, Renderer> DynamicTextInput<'a, Message, Theme, Renderer>
where
    Renderer: 'a + text::Renderer,
    Message: 'a + Clone,
    Theme: 'a + text_input::Catalog + iced::widget::text::Catalog,
{
    pub fn new(placeholder: &str, value: &str) -> Self
    where
        <Theme as iced::widget::text::Catalog>::Class<'a>:
        From<iced::widget::text::StyleFn<'a, Theme>>,
    {
        let input = TextInput::new(placeholder, value).padding(0);

        let label = if !value.is_empty() {
            Text::new(value.to_owned())
                .wrapping(Wrapping::None)
                .shaping(Shaping::Advanced)
        } else {
            Text::new(placeholder.to_owned())
                .wrapping(Wrapping::None)
                .shaping(Shaping::Advanced)
                .style(|theme| iced::widget::text::Style {
                    color: Some(get_placeholder_color(theme)),
                })
        };

        DynamicTextInput {
            content_input: input,
            content_label: label,

            width: Length::Fill,
            interaction: None,

            on_enter: None,
            on_submit: None,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        let width = width.into();
        self.width = width;
        self.content_input = self.content_input.width(width);
        self.content_label = self.content_label.width(width);

        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        let size = size.into();
        self.content_input = self.content_input.size(size);
        self.content_label = self.content_label.size(size);

        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        let line_height = line_height.into();
        self.content_input = self.content_input.line_height(line_height);
        self.content_label = self.content_label.line_height(line_height);

        self
    }

    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        let alignment = alignment.into();
        self.content_input = self.content_input.align_x(alignment);
        self.content_label = self.content_label.align_x(alignment);

        self
    }

    pub fn on_change(mut self, on_change: impl Fn(String) -> Message + 'a) -> Self {
        self.content_input = self.content_input.on_input(on_change);
        self
    }

    pub fn on_enter(mut self, on_enter: Message) -> Self {
        self.on_enter = Some(on_enter);
        self
    }

    pub fn on_submit(mut self, on_submit: Message) -> Self {
        self.on_submit = Some(on_submit);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for DynamicTextInput<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
    Message: Clone,
    Theme: text_input::Catalog + iced::widget::text::Catalog,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state: &State = tree.state.downcast_ref();

        self.get_widget_mut(state.is_edit_mode).layout(
            &mut tree.children[state.get_child_index()],
            renderer,
            limits,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state: &State = tree.state.downcast_ref();

        self.get_widget(state.is_edit_mode).draw(
            &tree.children[state.get_child_index()],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.content_label as &dyn Widget<Message, Theme, Renderer>),
            Tree::new(&self.content_input as &dyn Widget<Message, Theme, Renderer>),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[
            &self.content_label as &dyn Widget<Message, Theme, Renderer>,
            &self.content_input as &dyn Widget<Message, Theme, Renderer>,
        ]);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state: &State = tree.state.downcast_ref();
        self.get_widget_mut(state.is_edit_mode).operate(
            &mut tree.children[state.get_child_index()],
            layout,
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state: &mut State = tree.state.downcast_mut();
        let content = self.get_widget_mut(state.is_edit_mode);

        content.update(
            &mut tree.children[state.get_child_index()],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if state.is_edit_mode {
            let input_state: &mut text_input::State<Renderer::Paragraph> =
                tree.children[state.get_child_index()].state.downcast_mut();

            if input_state.is_focused()
                && let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event.clone()
                && key == Key::Named(Named::Enter)
            {
                input_state.unfocus();
                shell.capture_event();
            }

            if !input_state.is_focused() {
                state.is_edit_mode = false;

                if let Some(message) = self.on_submit.clone() {
                    shell.publish(message);
                }

                shell.invalidate_layout();
                shell.request_redraw();
            }
        }

        handle_enter_event::<Message, Renderer>(
            tree,
            event,
            layout,
            cursor,
            shell,
            self.on_enter.clone(),
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state: &State = tree.state.downcast_ref();

        let content_interaction = self.get_widget(state.is_edit_mode).mouse_interaction(
            &tree.children[state.get_child_index()],
            layout,
            cursor,
            viewport,
            renderer,
        );

        match (self.interaction, content_interaction) {
            (Some(interaction), mouse::Interaction::None) if cursor.is_over(layout.bounds()) => {
                interaction
            }
            _ => content_interaction,
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state: &State = tree.state.downcast_ref();

        self.get_widget_mut(state.is_edit_mode).overlay(
            &mut tree.children[state.get_child_index()],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<DynamicTextInput<'a, Message, Theme, Renderer>>
for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + text::Renderer,
    Theme: text_input::Catalog + iced::widget::text::Catalog,
{
    fn from(
        area: DynamicTextInput<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(area)
    }
}

fn handle_enter_event<Message: Clone, Renderer: text::Renderer>(
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    on_enter: Option<Message>,
) {
    let state: &mut State = tree.state.downcast_mut();

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let bounds = layout.bounds();

            if cursor.is_over(bounds) {
                let state = tree.state.downcast_mut::<State>();

                state.is_pressed = true;

                shell.capture_event();
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) => {
            if state.is_pressed {
                state.is_pressed = false;

                let bounds = layout.bounds();

                if cursor.is_over(bounds)
                    && let Some(cursor_position) = cursor.position()
                {
                    let new_click = mouse::Click::new(
                        cursor_position,
                        mouse::Button::Left,
                        state.previous_click,
                    );

                    state.previous_click = Some(new_click);

                    if matches!(new_click.kind(), mouse::click::Kind::Double) {
                        enter_edit_mode::<Message, Renderer>(tree, shell, on_enter);
                    }
                }

                shell.capture_event();
            }
        }
        Event::Touch(touch::Event::FingerLost { .. }) => {
            state.is_pressed = false;
        }
        _ => {}
    }
}

fn enter_edit_mode<Message: Clone, Renderer: text::Renderer>(
    tree: &mut Tree,
    shell: &mut Shell<'_, Message>,
    on_enter: Option<Message>,
) {
    let state: &mut State = tree.state.downcast_mut();
    state.is_edit_mode = true;

    if let Some(message) = on_enter {
        shell.publish(message);
    }

    shell.invalidate_layout();

    let input_state: &mut text_input::State<Renderer::Paragraph> =
        tree.children[state.get_child_index()].state.downcast_mut();

    input_state.focus();
    input_state.select_all();

    shell.request_redraw();
}
