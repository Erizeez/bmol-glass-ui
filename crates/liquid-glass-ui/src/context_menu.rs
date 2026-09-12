//! Authentic Apple-style Context Menu & Popover component.
//!
//! Provides a polished, physical glass-surfaced context menu matching macOS
//! Human Interface Guidelines (HIG):
//! - Continuous 8.0 pt squircle container curvature
//! - Translucent frosted glass panel with 1px fine edge rim & deep elevation drop shadows
//! - Smooth Accent hover pill with crisp white text inversion
//! - Full support for shortcuts, icons, checkable states, sections, separators, and destructive actions

use bmol_designs::menu_metrics;
use iced::{
    Alignment, Background, Border, Color, Element, Length, Padding, Shadow, Theme, Vector,
    font::Weight,
    widget::{button, column, container, row, space, text},
};

use crate::{
    components::{ComponentRenderer, icon_tinted},
    font,
    icon::UiIcon,
    theme::{UiColorScheme, UiPalette, UiTheme},
};

/// A single item entry within a [`ContextMenu`].
#[derive(Debug, Clone)]
pub enum MenuItem<Message> {
    /// A clickable action item with optional shortcut, icon, and destructive styling.
    Action {
        label: String,
        shortcut: Option<String>,
        icon: Option<UiIcon>,
        destructive: bool,
        disabled: bool,
        on_press: Option<Message>,
    },
    /// A toggleable item displaying a leading checkmark when active.
    Checkbox { label: String, checked: bool, disabled: bool, on_toggle: Option<Message> },
    /// A submenu item with a trailing disclosure chevron indicating hierarchy.
    Submenu { label: String, icon: Option<UiIcon>, disabled: bool, on_hover: Option<Message> },
    /// An informational non-clickable section header.
    Section(String),
    /// A subtle divider line separating logical item groups.
    Separator,
}

impl<Message: Clone> MenuItem<Message> {
    /// Creates a standard clickable action item.
    pub fn action(label: impl Into<String>) -> Self {
        Self::Action {
            label: label.into(),
            shortcut: None,
            icon: None,
            destructive: false,
            disabled: false,
            on_press: None,
        }
    }

    /// Creates a checkable toggle item.
    pub fn checkbox(label: impl Into<String>, checked: bool) -> Self {
        Self::Checkbox { label: label.into(), checked, disabled: false, on_toggle: None }
    }

    /// Creates a submenu item pointing to an expanded submenu.
    pub fn submenu(label: impl Into<String>) -> Self {
        Self::Submenu { label: label.into(), icon: None, disabled: false, on_hover: None }
    }

    /// Creates an informational section header label.
    pub fn section(label: impl Into<String>) -> Self {
        Self::Section(label.into())
    }

    /// Creates a thin 1px horizontal separator line.
    #[must_use]
    pub const fn separator() -> Self {
        Self::Separator
    }

    /// Attaches a keyboard shortcut string (e.g. `"⌘C"`, `"⇧⌘N"`).
    #[must_use]
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        if let Self::Action { shortcut: ref mut s, .. } = self {
            *s = Some(shortcut.into());
        }
        self
    }

    /// Attaches an icon displayed in the leading slot.
    #[must_use]
    pub fn with_icon(mut self, icon: UiIcon) -> Self {
        match self {
            Self::Action { icon: ref mut i, .. } | Self::Submenu { icon: ref mut i, .. } => {
                *i = Some(icon);
            }
            _ => {}
        }
        self
    }

    /// Marks the item as a destructive action (rendered in red with red hover highlight).
    #[must_use]
    pub fn with_destructive(mut self, is_destructive: bool) -> Self {
        if let Self::Action { ref mut destructive, .. } = self {
            *destructive = is_destructive;
        }
        self
    }

    /// Marks the item as disabled (non-interactive, visually muted).
    #[must_use]
    pub fn with_disabled(mut self, is_disabled: bool) -> Self {
        match self {
            Self::Action { ref mut disabled, .. }
            | Self::Checkbox { ref mut disabled, .. }
            | Self::Submenu { ref mut disabled, .. } => {
                *disabled = is_disabled;
            }
            _ => {}
        }
        self
    }

    /// Sets the message emitted when the action item is clicked.
    #[must_use]
    pub fn on_press(mut self, msg: Message) -> Self {
        if let Self::Action { ref mut on_press, .. } = self {
            *on_press = Some(msg);
        }
        self
    }

    /// Sets the message emitted when the checkbox item is clicked.
    #[must_use]
    pub fn on_toggle(mut self, msg: Message) -> Self {
        if let Self::Checkbox { ref mut on_toggle, .. } = self {
            *on_toggle = Some(msg);
        }
        self
    }
}

/// A floating Apple-style Context Menu container.
#[derive(Debug, Clone)]
pub struct ContextMenu<Message> {
    items: Vec<MenuItem<Message>>,
    width: Option<f32>,
    min_width: f32,
    scheme_override: Option<UiColorScheme>,
    show_border: bool,
    transparent_background: bool,
}

impl<Message: Clone + 'static> Default for ContextMenu<Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Message: Clone + 'static> ContextMenu<Message> {
    /// Creates an empty context menu with standard defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            width: Some(menu_metrics::DEFAULT_WIDTH),
            min_width: menu_metrics::MIN_WIDTH,
            scheme_override: None,
            show_border: true,
            transparent_background: false,
        }
    }

    /// Appends a single [`MenuItem`] to the menu.
    #[must_use]
    pub fn item(mut self, item: MenuItem<Message>) -> Self {
        self.items.push(item);
        self
    }

    /// Appends multiple [`MenuItem`]s to the menu.
    #[must_use]
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem<Message>>) -> Self {
        self.items.extend(items);
        self
    }

    /// Overrides the fixed width of the menu in logical points.
    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides the minimum width of the menu.
    #[must_use]
    pub const fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Explicitly forces a specific [`UiColorScheme`] instead of deriving from the active theme.
    #[must_use]
    pub const fn with_scheme(mut self, scheme: UiColorScheme) -> Self {
        self.scheme_override = Some(scheme);
        self
    }

    /// Toggles whether the outer container border and shadow should be drawn.
    ///
    /// Setting to `false` allows embedding the context menu inside custom canvas
    /// shapes (e.g. popover cards with protruding arrows) without conflicting border outlines.
    #[must_use]
    pub const fn with_border(mut self, show_border: bool) -> Self {
        self.show_border = show_border;
        self
    }

    /// Toggles whether the container background should be transparent.
    ///
    /// Useful when the physical glass material is rendered by an underlying canvas layer.
    #[must_use]
    pub const fn with_transparent_background(mut self, transparent: bool) -> Self {
        self.transparent_background = transparent;
        self
    }

    /// Returns the number of items in the menu.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the menu contains no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns a slice of the menu items.
    #[must_use]
    pub fn items_slice(&self) -> &[MenuItem<Message>] {
        &self.items
    }

    /// Builds the menu element tree.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn view<'a, R>(&'a self, theme: &Theme) -> Element<'a, Message, Theme, R>
    where
        R: ComponentRenderer + 'a,
    {
        let (scheme, palette) = if let Some(scheme) = self.scheme_override {
            (scheme, UiTheme::new(scheme).palette())
        } else {
            let t = UiTheme::from_iced(theme);
            (t.scheme(), t.palette())
        };
        let is_dark = scheme == UiColorScheme::Dark;

        let has_any_leading = self.items.iter().any(|item| match item {
            MenuItem::Checkbox { .. } => true,
            MenuItem::Action { icon, .. } | MenuItem::Submenu { icon, .. } => icon.is_some(),
            _ => false,
        });

        let mut list = column![].spacing(1.0);

        for item in &self.items {
            match item {
                MenuItem::Action { label, shortcut, icon, destructive, disabled, on_press } => {
                    let is_destructive = *destructive;
                    let is_disabled = *disabled;
                    let action_msg = on_press.clone();

                    let btn = button(Self::render_row::<R>(
                        label,
                        shortcut.as_deref(),
                        *icon,
                        false,
                        false,
                        has_any_leading,
                        is_destructive,
                        is_disabled,
                        &palette,
                    ))
                    .padding(Padding {
                        top: 0.0,
                        right: menu_metrics::ITEM_HORIZONTAL_PADDING,
                        bottom: 0.0,
                        left: menu_metrics::ITEM_HORIZONTAL_PADDING,
                    })
                    .width(Length::Fill)
                    .height(Length::Fixed(menu_metrics::ITEM_HEIGHT))
                    .style(move |_theme, status| {
                        Self::pill_style(status, is_destructive, is_disabled, &palette)
                    });

                    let btn = if is_disabled {
                        btn
                    } else if let Some(msg) = action_msg {
                        btn.on_press(msg)
                    } else {
                        btn
                    };

                    list = list.push(btn);
                }
                MenuItem::Checkbox { label, checked, disabled, on_toggle } => {
                    let is_checked = *checked;
                    let is_disabled = *disabled;
                    let toggle_msg = on_toggle.clone();

                    let btn = button(Self::render_row::<R>(
                        label,
                        None,
                        None,
                        is_checked,
                        false,
                        has_any_leading,
                        false,
                        is_disabled,
                        &palette,
                    ))
                    .padding(Padding {
                        top: 0.0,
                        right: menu_metrics::ITEM_HORIZONTAL_PADDING,
                        bottom: 0.0,
                        left: menu_metrics::ITEM_HORIZONTAL_PADDING,
                    })
                    .width(Length::Fill)
                    .height(Length::Fixed(menu_metrics::ITEM_HEIGHT))
                    .style(move |_theme, status| {
                        Self::pill_style(status, false, is_disabled, &palette)
                    });

                    let btn = if is_disabled {
                        btn
                    } else if let Some(msg) = toggle_msg {
                        btn.on_press(msg)
                    } else {
                        btn
                    };

                    list = list.push(btn);
                }
                MenuItem::Submenu { label, icon, disabled, on_hover: _ } => {
                    let is_disabled = *disabled;

                    let btn = button(Self::render_row::<R>(
                        label,
                        None,
                        *icon,
                        false,
                        true,
                        has_any_leading,
                        false,
                        is_disabled,
                        &palette,
                    ))
                    .padding(Padding {
                        top: 0.0,
                        right: menu_metrics::ITEM_HORIZONTAL_PADDING,
                        bottom: 0.0,
                        left: menu_metrics::ITEM_HORIZONTAL_PADDING,
                    })
                    .width(Length::Fill)
                    .height(Length::Fixed(menu_metrics::ITEM_HEIGHT))
                    .style(move |_theme, status| {
                        Self::pill_style(status, false, is_disabled, &palette)
                    });

                    list = list.push(btn);
                }
                MenuItem::Section(title) => {
                    let header = container(
                        text(title.as_str())
                            .size(font::size::CAPTION)
                            .font(font::ui_font(Weight::Semibold))
                            .color(palette.text_tertiary),
                    )
                    .padding(Padding {
                        top: 4.0,
                        right: menu_metrics::ITEM_HORIZONTAL_PADDING,
                        bottom: 2.0,
                        left: menu_metrics::ITEM_HORIZONTAL_PADDING,
                    })
                    .height(Length::Fixed(menu_metrics::SECTION_HEADER_HEIGHT));

                    list = list.push(header);
                }
                MenuItem::Separator => {
                    let sep_color = if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.09)
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                    };

                    let line = container(space())
                        .height(Length::Fixed(menu_metrics::SEPARATOR_HEIGHT))
                        .width(Length::Fill)
                        .style(move |_theme| container::Style {
                            background: Some(Background::Color(sep_color)),
                            ..container::Style::default()
                        });

                    let pad = container(line)
                        .padding(Padding {
                            top: menu_metrics::SEPARATOR_MARGIN_V,
                            right: 2.0,
                            bottom: menu_metrics::SEPARATOR_MARGIN_V,
                            left: 2.0,
                        })
                        .width(Length::Fill);

                    list = list.push(pad);
                }
            }
        }

        let container_w = self.width.unwrap_or(menu_metrics::DEFAULT_WIDTH).max(self.min_width);

        let show_border = self.show_border;
        let transparent_bg = self.transparent_background;

        container(list)
            .width(Length::Fixed(container_w))
            .padding(Padding::from([
                menu_metrics::CONTAINER_PADDING,
                menu_metrics::CONTAINER_PADDING,
            ]))
            .style(move |_theme| {
                if is_dark {
                    let (r, g, b, a) = menu_metrics::DARK_MENU_BASE_RGBA_F32;
                    let background = if transparent_bg {
                        None
                    } else {
                        Some(Background::Color(Color::from_rgba(r, g, b, a)))
                    };
                    let (border, shadow) = if show_border {
                        (
                            Border::default()
                                .rounded(menu_metrics::CONTAINER_CORNER_RADIUS)
                                .width(1.0)
                                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.18)),
                            Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.65),
                                offset: Vector::new(0.0, 14.0),
                                blur_radius: 36.0,
                            },
                        )
                    } else {
                        (Border::default(), Shadow::default())
                    };

                    container::Style { background, border, shadow, ..container::Style::default() }
                } else {
                    let (r, g, b, a) = menu_metrics::LIGHT_MENU_BASE_RGBA_F32;
                    let background = if transparent_bg {
                        None
                    } else {
                        Some(Background::Color(Color::from_rgba(r, g, b, a)))
                    };
                    let (border, shadow) = if show_border {
                        (
                            Border::default()
                                .rounded(menu_metrics::CONTAINER_CORNER_RADIUS)
                                .width(1.0)
                                .color(Color::from_rgba(0.0, 0.0, 0.0, 0.12)),
                            Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.22),
                                offset: Vector::new(0.0, 12.0),
                                blur_radius: 32.0,
                            },
                        )
                    } else {
                        (Border::default(), Shadow::default())
                    };

                    container::Style { background, border, shadow, ..container::Style::default() }
                }
            })
            .into()
    }

    fn pill_style(
        status: button::Status,
        destructive: bool,
        disabled: bool,
        palette: &UiPalette,
    ) -> button::Style {
        if disabled {
            return button::Style {
                background: None,
                text_color: palette.text_tertiary,
                border: Border::default(),
                ..button::Style::default()
            };
        }

        match status {
            button::Status::Hovered | button::Status::Pressed => {
                let bg_color = if destructive {
                    Color::from_rgba(1.0, 0.27, 0.23, 0.95)
                } else {
                    palette.accent
                };

                button::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: Color::WHITE,
                    border: Border::default().rounded(menu_metrics::ITEM_HIGHLIGHT_RADIUS),
                    ..button::Style::default()
                }
            }
            _ => {
                let text_color = if destructive {
                    Color::from_rgba(1.0, 0.27, 0.23, 0.95)
                } else {
                    palette.text_primary
                };

                button::Style {
                    background: None,
                    text_color,
                    border: Border::default(),
                    ..button::Style::default()
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn render_row<'a, R>(
        label: &str,
        shortcut: Option<&str>,
        icon: Option<UiIcon>,
        checked: bool,
        is_submenu: bool,
        has_leading: bool,
        destructive: bool,
        disabled: bool,
        palette: &UiPalette,
    ) -> Element<'a, Message, Theme, R>
    where
        R: ComponentRenderer + 'a,
    {
        let mut r = row![].align_y(Alignment::Center).width(Length::Fill);

        // 1. Leading Slot: checkmark, icon, or placeholder
        if has_leading {
            if checked {
                let check_element = text("✓").size(13.0).font(font::ui_font(Weight::Bold));
                r = r.push(
                    container(check_element)
                        .width(Length::Fixed(menu_metrics::LEADING_SLOT_WIDTH))
                        .align_x(iced::alignment::Horizontal::Left),
                );
            } else if let Some(icon_asset) = icon {
                let icon_color = if disabled {
                    palette.text_tertiary
                } else if destructive {
                    Color::from_rgba(1.0, 0.27, 0.23, 0.95)
                } else {
                    palette.text_secondary
                };
                let icon_widget =
                    icon_tinted(icon_asset, menu_metrics::ICON_SIZE, move |_| icon_color);
                r = r.push(
                    container(icon_widget)
                        .width(Length::Fixed(menu_metrics::LEADING_SLOT_WIDTH))
                        .align_x(iced::alignment::Horizontal::Left),
                );
            } else {
                r = r.push(space().width(Length::Fixed(menu_metrics::LEADING_SLOT_WIDTH)));
            }
        }

        // 2. Primary Label (inherits text_color from button::Style)
        r = r.push(
            text(label.to_string()).size(font::size::BODY).font(font::ui_font(Weight::Normal)),
        );

        // 3. Flexible Horizontal Gap
        r = r.push(space().width(Length::Fill));

        // 4. Trailing Slot: Shortcut or Submenu Chevron
        if let Some(sc) = shortcut {
            let sc_color = if disabled { palette.text_tertiary } else { palette.text_secondary };

            r = r.push(
                text(sc.to_string())
                    .size(font::size::CAPTION)
                    .font(font::ui_font(Weight::Normal))
                    .color(sc_color),
            );
        } else if is_submenu {
            let chevron_color =
                if disabled { palette.text_tertiary } else { palette.text_secondary };
            let chevron = icon_tinted(UiIcon::ChevronRight, 10.0, move |_| chevron_color);
            r = r.push(chevron);
        }

        r.into()
    }
}

/// Helper function to build and render a context menu with standard items.
#[must_use]
pub fn view_context_menu<'a, Message, R>(
    menu: &'a ContextMenu<Message>,
    theme: &Theme,
) -> Element<'a, Message, Theme, R>
where
    Message: Clone + 'static,
    R: ComponentRenderer + 'a,
{
    menu.view(theme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Renderer;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestMsg {
        Cut,
        Copy,
        Paste,
        Delete,
        ToggleLineNumbers,
    }

    #[test]
    fn test_menu_item_builders() {
        let cut = MenuItem::action("Settings")
            .with_shortcut("⌘,")
            .with_icon(UiIcon::Gear)
            .on_press(TestMsg::Cut);

        if let MenuItem::Action { label, shortcut, icon, destructive, disabled, on_press } = cut {
            assert_eq!(label, "Settings");
            assert_eq!(shortcut.as_deref(), Some("⌘,"));
            assert_eq!(icon, Some(UiIcon::Gear));
            assert!(!destructive);
            assert!(!disabled);
            assert_eq!(on_press, Some(TestMsg::Cut));
        } else {
            panic!("Expected Action variant");
        }

        let delete = MenuItem::action("Delete")
            .with_destructive(true)
            .with_disabled(true)
            .on_press(TestMsg::Delete);

        if let MenuItem::Action { destructive, disabled, .. } = delete {
            assert!(destructive);
            assert!(disabled);
        } else {
            panic!("Expected Action variant");
        }

        let chk =
            MenuItem::checkbox("Show Line Numbers", true).on_toggle(TestMsg::ToggleLineNumbers);

        if let MenuItem::Checkbox { label, checked, on_toggle, .. } = chk {
            assert_eq!(label, "Show Line Numbers");
            assert!(checked);
            assert_eq!(on_toggle, Some(TestMsg::ToggleLineNumbers));
        } else {
            panic!("Expected Checkbox variant");
        }

        let sub = MenuItem::<TestMsg>::submenu("Share");
        if let MenuItem::Submenu { label, .. } = sub {
            assert_eq!(label, "Share");
        } else {
            panic!("Expected Submenu variant");
        }

        let sec = MenuItem::<TestMsg>::section("EDITING");
        if let MenuItem::Section(title) = sec {
            assert_eq!(title, "EDITING");
        } else {
            panic!("Expected Section variant");
        }

        let sep = MenuItem::<TestMsg>::separator();
        assert!(matches!(sep, MenuItem::Separator));
    }

    #[test]
    fn test_context_menu_composition_and_rendering() {
        let menu: ContextMenu<TestMsg> = ContextMenu::new()
            .width(240.0)
            .min_width(200.0)
            .item(MenuItem::section("CLIPBOARD"))
            .item(MenuItem::action("Cut").with_shortcut("⌘X").on_press(TestMsg::Cut))
            .item(MenuItem::action("Copy").with_shortcut("⌘C").on_press(TestMsg::Copy))
            .item(MenuItem::action("Paste").with_shortcut("⌘V").on_press(TestMsg::Paste))
            .item(MenuItem::separator())
            .item(MenuItem::checkbox("Line Numbers", true).on_toggle(TestMsg::ToggleLineNumbers))
            .item(MenuItem::submenu("Share"))
            .item(MenuItem::separator())
            .item(MenuItem::action("Delete").with_destructive(true).on_press(TestMsg::Delete));

        assert_eq!(menu.items.len(), 9);
        assert_eq!(menu.width, Some(240.0));
        assert_eq!(menu.min_width, 200.0);

        let light_elem: Element<'_, TestMsg, Theme, Renderer> = menu.view(&Theme::Light);
        drop(light_elem);

        let dark_elem: Element<'_, TestMsg, Theme, Renderer> = menu.view(&Theme::Dark);
        drop(dark_elem);

        let forced_menu = menu.clone().with_scheme(UiColorScheme::Light);
        let forced_light: Element<'_, TestMsg, Theme, Renderer> = forced_menu.view(&Theme::Dark);
        drop(forced_light);
    }

    #[test]
    fn test_context_menu_calibrated_colors() {
        let (dark_r, _, _, dark_a) = menu_metrics::DARK_MENU_BASE_RGBA_F32;
        assert_eq!((dark_r * 255.0 * dark_a).round() as u8, 33);
        assert_eq!((dark_r * 255.0 * dark_a + 255.0 * (1.0 - dark_a)).round() as u8, 86);

        let (light_r, _, _, light_a) = menu_metrics::LIGHT_MENU_BASE_RGBA_F32;
        assert_eq!((light_r * 255.0 * light_a).round() as u8, 185);
        assert_eq!((light_r * 255.0 * light_a + 255.0 * (1.0 - light_a)).round() as u8, 255);
    }
}
