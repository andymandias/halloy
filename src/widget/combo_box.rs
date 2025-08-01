use std::cell::RefCell;
use std::fmt::Display;
use std::time::Instant;

use iced::advanced::graphics::text::Paragraph;
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, text,
    widget,
};
use iced::overlay::menu;
use iced::widget::text::LineHeight;
use iced::widget::{TextInput, text_input};
use iced::{Event, Length, Padding, Rectangle, Vector, keyboard, window};

use super::Element;
use crate::Theme;

/// A widget for searching and selecting a single value from a list of options.
///
/// This widget is composed by a [`TextInput`] that can be filled with the text
/// to search for corresponding values from the list of options that are displayed
/// as a [`Menu`].
pub struct ComboBox<
    'a,
    T,
    Message,
    Theme = crate::Theme,
    Renderer = super::Renderer,
> where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    state: &'a State<T>,
    text_input: TextInput<'a, TextInputEvent, Theme, Renderer>,
    font: Option<Renderer::Font>,
    selection: text_input::Value,
    on_selected: Box<dyn Fn(T) -> Message>,
    on_option_hovered: Option<Box<dyn Fn(T) -> Message>>,
    on_close: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message>>,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
    padding: Padding,
    size: Option<f32>,
}

impl<'a, T, Message, Theme, Renderer> ComboBox<'a, T, Message, Theme, Renderer>
where
    T: std::fmt::Display + Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new [`ComboBox`] with the given list of options, a placeholder,
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
            font: None,
            selection: text_input::Value::new(&selection),
            on_selected: Box::new(on_selected),
            on_option_hovered: None,
            on_input: None,
            on_close: None,
            menu_class: <Theme as Catalog>::default_menu(),
            padding: text_input::DEFAULT_PADDING,
            size: None,
        }
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`] of the [`ComboBox`].
    pub fn on_input(
        mut self,
        on_input: impl Fn(String) -> Message + 'static,
    ) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that will be produced when an option of the
    /// [`ComboBox`] is hovered using the arrow keys.
    pub fn on_option_hovered(
        mut self,
        on_selection: impl Fn(T) -> Message + 'static,
    ) -> Self {
        self.on_option_hovered = Some(Box::new(on_selection));
        self
    }

    /// Sets the message that will be produced when the outside area
    /// of the [`ComboBox`] is pressed.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the [`Padding`] of the [`ComboBox`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self.text_input = self.text_input.padding(self.padding);
        self
    }

    /// Sets the [`Font`] of the [`ComboBox`].
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.text_input = self.text_input.font(font);
        self.font = Some(font);
        self
    }

    /// Sets the [`Icon`] of the [`ComboBox`].
    pub fn icon(mut self, icon: text_input::Icon<Renderer::Font>) -> Self {
        self.text_input = self.text_input.icon(icon);
        self
    }

    /// Returns whether the [`ComboBox`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.state.is_focused()
    }

    /// Sets the text sixe of the [`ComboBox`].
    pub fn size(mut self, size: f32) -> Self {
        self.text_input = self.text_input.size(size);
        self.size = Some(size);
        self
    }

    /// Sets the [`LineHeight`] of the [`ComboBox`].
    pub fn line_height(self, line_height: impl Into<LineHeight>) -> Self {
        Self {
            text_input: self.text_input.line_height(line_height),
            ..self
        }
    }

    /// Sets the width of the [`ComboBox`].
    pub fn width(self, width: impl Into<Length>) -> Self {
        Self {
            text_input: self.text_input.width(width),
            ..self
        }
    }

    /// Sets the style of the input of the [`ComboBox`].
    #[must_use]
    pub fn input_style(
        mut self,
        style: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self
    where
        <Theme as text_input::Catalog>::Class<'a>:
            From<text_input::StyleFn<'a, Theme>>,
    {
        self.text_input = self.text_input.style(style);
        self
    }

    /// Sets the style of the menu of the [`ComboBox`].
    #[must_use]
    pub fn menu_style(
        mut self,
        style: impl Fn(&Theme) -> menu::Style + 'a,
    ) -> Self
    where
        <Theme as menu::Catalog>::Class<'a>: From<menu::StyleFn<'a, Theme>>,
    {
        self.menu_class = (Box::new(style) as menu::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the input of the [`ComboBox`].
    #[must_use]
    pub fn input_class(
        mut self,
        class: impl Into<<Theme as text_input::Catalog>::Class<'a>>,
    ) -> Self {
        self.text_input = self.text_input.class(class);
        self
    }

    /// Sets the style class of the menu of the [`ComboBox`].
    #[must_use]
    pub fn menu_class(
        mut self,
        class: impl Into<<Theme as menu::Catalog>::Class<'a>>,
    ) -> Self {
        self.menu_class = class.into();
        self
    }
}

/// The local state of a [`ComboBox`].
#[derive(Debug, Clone)]
pub struct State<T>(RefCell<Inner<T>>);

#[derive(Debug, Clone)]
struct Inner<T> {
    text_input: text_input::State<Paragraph>,
    value: String,
    options: Vec<T>,
    option_matchers: Vec<String>,
    filtered_options: Filtered<T>,
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
    /// Creates a new [`State`] for a [`ComboBox`] with the given list of options.
    pub fn new(options: Vec<T>) -> Self {
        Self::with_selection(options, None)
    }

    /// Creates a new [`State`] for a [`ComboBox`] with the given list of options
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

        Self(RefCell::new(Inner {
            text_input: text_input::State::new(),
            value,
            options,
            option_matchers,
            filtered_options,
        }))
    }

    /// Focuses the [`ComboBox`].
    pub fn focused(self) -> Self {
        self.focus();
        self
    }

    /// Focuses the [`ComboBox`].
    pub fn focus(&self) {
        let mut inner = self.0.borrow_mut();

        inner.text_input.focus();
    }

    /// Unfocuses the [`ComboBox`].
    pub fn unfocus(&self) {
        let mut inner = self.0.borrow_mut();

        inner.text_input.unfocus();
    }

    /// Returns whether the [`ComboBox`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        let inner = self.0.borrow();

        inner.text_input.is_focused()
    }

    fn value(&self) -> String {
        let inner = self.0.borrow();

        inner.value.clone()
    }

    fn text_input_tree(&self) -> widget::Tree {
        let inner = self.0.borrow();

        inner.text_input_tree()
    }

    fn update_text_input(&self, tree: widget::Tree) {
        let mut inner = self.0.borrow_mut();

        inner.update_text_input(tree);
    }

    fn with_inner<O>(&self, f: impl FnOnce(&Inner<T>) -> O) -> O {
        let inner = self.0.borrow();

        f(&inner)
    }

    fn with_inner_mut(&self, f: impl FnOnce(&mut Inner<T>)) {
        let mut inner = self.0.borrow_mut();

        f(&mut inner);
    }

    fn sync_filtered_options(&self, options: &mut Filtered<T>) {
        let inner = self.0.borrow();

        inner.filtered_options.sync(options);
    }
}

impl<T> Inner<T> {
    fn text_input_tree(&self) -> widget::Tree {
        widget::Tree {
            tag: widget::tree::Tag::of::<text_input::State<Paragraph>>(),
            state: widget::tree::State::new(self.text_input.clone()),
            children: vec![],
        }
    }

    fn update_text_input(&mut self, tree: widget::Tree) {
        self.text_input = tree
            .state
            .downcast_ref::<text_input::State<Paragraph>>()
            .clone();
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

    fn empty() -> Self {
        Self {
            options: vec![],
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

impl<'a, T, Message, Renderer> Widget<Message, Theme, Renderer>
    for ComboBox<'a, T, Message, Theme, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone,
    Renderer: text::Renderer,
    Theme: 'a,
{
    fn size(&self) -> iced::Size<Length> {
        Widget::<TextInputEvent, Theme, Renderer>::size(&self.text_input)
    }

    fn size_hint(&self) -> iced::Size<Length> {
        Widget::<TextInputEvent, Theme, Renderer>::size_hint(&self.text_input)
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut tree = self.state.text_input_tree();
        let node = Widget::<TextInputEvent, Theme, Renderer>::layout(
            &self.text_input,
            &mut tree,
            renderer,
            limits,
        );
        self.state.update_text_input(tree);
        node
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Menu<T>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Menu::<T> {
            menu: menu::State::new(),
            filtered_options: Filtered::empty(),
            hovered_option: Some(0),
            new_selection: None,
        })
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
        let menu = tree.state.downcast_mut::<Menu<T>>();

        let started_focused = self.state.is_focused();
        // This is intended to check whether or not the message buffer was empty,
        // since `Shell` does not expose such functionality.
        let mut published_message_to_shell = false;

        // Create a new list of local messages
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        // Provide it to the widget
        let mut tree = self.state.text_input_tree();
        self.text_input.update(
            &mut tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );
        self.state.update_text_input(tree);

        if local_shell.is_event_captured() {
            shell.capture_event();
        }

        match local_shell.redraw_request() {
            window::RedrawRequest::NextFrame => shell.request_redraw(),
            window::RedrawRequest::At(at) => shell.request_redraw_at(at),
            window::RedrawRequest::Wait => {}
        }

        // Then finally react to them here
        for message in local_messages {
            let TextInputEvent::TextChanged(new_value) = message;

            if let Some(on_input) = &self.on_input {
                shell.publish((on_input)(new_value.clone()));
            }

            // Couple the filtered options with the `ComboBox`
            // value and only recompute them when the value changes,
            // instead of doing it in every `view` call
            self.state.with_inner_mut(|state| {
                menu.hovered_option = Some(0);
                state.value = new_value;

                state.filtered_options.update(
                    search(
                        &state.options,
                        &state.option_matchers,
                        &state.value,
                    )
                    .cloned()
                    .collect(),
                );
            });
            shell.invalidate_layout();
            shell.request_redraw();
        }

        if self.state.is_focused() {
            self.state.with_inner(|state| {
                if !started_focused
                    && let Some(on_option_hovered) = &mut self.on_option_hovered
                    {
                        let hovered_option = menu.hovered_option.unwrap_or(0);

                        if let Some(option) =
                            state.filtered_options.options.get(hovered_option)
                        {
                            shell.publish(on_option_hovered(option.clone()));
                            published_message_to_shell = true;
                        }
                    }

                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) = event
                {
                    let shift_modifier = modifiers.shift();

                    match (key, shift_modifier) {
                        (
                            keyboard::Key::Named(keyboard::key::Named::Enter),
                            _,
                        ) => {
                            if let Some(index) = &menu.hovered_option
                                && let Some(option) =
                                    state.filtered_options.options.get(*index)
                                {
                                    menu.new_selection = Some(option.clone());
                                }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        (
                            keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                            _,
                        )
                        | (
                            keyboard::Key::Named(keyboard::key::Named::Tab),
                            true,
                        ) => {
                            if let Some(index) = &mut menu.hovered_option {
                                if *index == 0 {
                                    *index = state
                                        .filtered_options
                                        .options
                                        .len()
                                        .saturating_sub(1);
                                } else {
                                    *index = index.saturating_sub(1);
                                }
                            } else {
                                menu.hovered_option = Some(0);
                            }

                            if let Some(on_option_selection) =
                                &mut self.on_option_hovered
                                && let Some(option) =
                                    menu.hovered_option.and_then(|index| {
                                        state
                                            .filtered_options
                                            .options
                                            .get(index)
                                    })
                                {
                                    // Notify the selection
                                    shell.publish((on_option_selection)(
                                        option.clone(),
                                    ));
                                    published_message_to_shell = true;
                                }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        (
                            keyboard::Key::Named(
                                keyboard::key::Named::ArrowDown,
                            ),
                            _,
                        )
                        | (
                            keyboard::Key::Named(keyboard::key::Named::Tab),
                            false,
                        ) => {
                            if let Some(index) = &mut menu.hovered_option {
                                if *index
                                    == state
                                        .filtered_options
                                        .options
                                        .len()
                                        .saturating_sub(1)
                                {
                                    *index = 0;
                                } else {
                                    *index = index.saturating_add(1).min(
                                        state
                                            .filtered_options
                                            .options
                                            .len()
                                            .saturating_sub(1),
                                    );
                                }
                            } else {
                                menu.hovered_option = Some(0);
                            }

                            if let Some(on_option_selection) =
                                &mut self.on_option_hovered
                                && let Some(option) =
                                    menu.hovered_option.and_then(|index| {
                                        state
                                            .filtered_options
                                            .options
                                            .get(index)
                                    })
                                {
                                    // Notify the selection
                                    shell.publish((on_option_selection)(
                                        option.clone(),
                                    ));
                                    published_message_to_shell = true;
                                }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        _ => {}
                    }
                }
            });
        }

        // If the overlay menu has selected something
        self.state.with_inner_mut(|state| {
            if let Some(selection) = menu.new_selection.take() {
                // Clear the value and reset the options and menu
                state.value = String::new();
                state.filtered_options.update(state.options.clone());
                menu.menu = menu::State::default();

                // Notify the selection
                shell.publish((self.on_selected)(selection));
                published_message_to_shell = true;

                // Unfocus the input
                let mut tree = state.text_input_tree();
                self.text_input.update(
                    &mut tree,
                    &Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    )),
                    layout,
                    mouse::Cursor::Unavailable,
                    renderer,
                    clipboard,
                    &mut Shell::new(&mut vec![]),
                    viewport,
                );
                state.update_text_input(tree);
            }
        });

        if started_focused
            && !self.state.is_focused()
            && !published_message_to_shell
            && let Some(message) = self.on_close.take() {
                shell.publish(message);
            }

        // Focus changed, invalidate widget tree to force a fresh `view`
        if started_focused != self.state.is_focused() {
            shell.invalidate_widgets();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let tree = self.state.text_input_tree();
        self.text_input
            .mouse_interaction(&tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let selection = if self.state.is_focused() || self.selection.is_empty()
        {
            None
        } else {
            Some(&self.selection)
        };

        let tree = self.state.text_input_tree();
        self.text_input
            .draw(&tree, renderer, theme, layout, cursor, selection, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let Menu {
            menu,
            filtered_options,
            hovered_option,
            ..
        } = tree.state.downcast_mut::<Menu<T>>();

        if self.state.is_focused() {
            let bounds = layout.bounds();

            self.state.sync_filtered_options(filtered_options);

            let mut menu = menu::Menu::new(
                menu,
                &filtered_options.options,
                hovered_option,
                |x| (self.on_selected)(x),
                self.on_option_hovered.as_deref(),
                &self.menu_class,
            )
            .width(bounds.width)
            .padding(self.padding);

            if let Some(font) = self.font {
                menu = menu.font(font);
            }

            if let Some(size) = self.size {
                menu = menu.text_size(size);
            }

            // Spacing between text_input and menu
            let spacing = 4.0;

            Some(menu.overlay(
                layout.position(),
                layout.bounds(),
                bounds.height + spacing,
            ))
        } else {
            None
        }
    }
}

impl<'a, T, Message> From<ComboBox<'a, T, Message>> for Element<'a, Message>
where
    T: Display + Clone + 'static,
    Message: 'a + Clone,
{
    fn from(combo_box: ComboBox<'a, T, Message>) -> Self {
        Self::new(combo_box)
    }
}

/// Search list of options for a given query.
pub fn search<'a, T, A>(
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

/// Build matchers from given list of options.
pub fn build_matchers<'a, T>(
    options: impl IntoIterator<Item = T> + 'a,
) -> Vec<String>
where
    T: Display + 'a,
{
    options
        .into_iter()
        .map(|opt| {
            let mut matcher = opt.to_string();
            matcher.retain(|c| c.is_ascii_alphanumeric());
            matcher.to_lowercase()
        })
        .collect()
}

pub fn combo_box<'a, T, Message>(
    state: &'a self::State<T>,
    placeholder: &str,
    selection: Option<&T>,
    on_selected: impl Fn(T) -> Message + 'static,
) -> ComboBox<'a, T, Message>
where
    T: std::fmt::Display + Clone,
{
    ComboBox::new(state, placeholder, selection, on_selected)
}

/// The theme catalog of a [`ComboBox`].
pub trait Catalog: text_input::Catalog + menu::Catalog {
    /// The default class for the text input of the [`ComboBox`].
    fn default_input<'a>() -> <Self as text_input::Catalog>::Class<'a> {
        <Self as text_input::Catalog>::default()
    }

    /// The default class for the menu of the [`ComboBox`].
    fn default_menu<'a>() -> <Self as menu::Catalog>::Class<'a> {
        <Self as menu::Catalog>::default()
    }
}

impl Catalog for iced::Theme {}
