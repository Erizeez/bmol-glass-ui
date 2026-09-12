//! Settings-style components modelled after macOS System Settings.
//!
//! Every component is themed through [`UiTheme`]/[`UiPalette`], so they adapt
//! to the active color scheme automatically, and generic over the app's
//! message type. Glass-surfaced controls pair a compositor [`GlassContainer`]
//! with a transparent foreground, matching the pattern used by the window
//! compositor.

// Style functions in this module are meant to be passed as `.style(fn)`
// references, so the pedantic `must_use` suggestion does not apply.
#![allow(clippy::must_use_candidate)]

use iced::{
    Alignment, Background, Border, Color, Element, Font, Length, Padding, Shadow, Theme, Vector,
    advanced::{
        Renderer as CoreRenderer, image as advanced_image, svg as advanced_svg,
        text::Renderer as TextRenderer,
    },
    widget::{
        button, column, container, pick_list, progress_bar, row, rule, scrollable, slider, space,
        text, text_input, toggler,
    },
};
use liquid_glass_scene::{Color as GlassColor, GlassId, GlassMaterial, Rect};

use crate::{
    GlassContainer, GlassForeground, GlassForegroundRenderer, GlassOverlay, GlassRole,
    UiColorScheme, UiCornerStyle, UiPalette, UiTheme, font,
    icon::{UiIcon, UiIconAsset, icon as ui_icon},
};

/// The renderer capabilities every component in this module needs.
pub trait ComponentRenderer:
    CoreRenderer
    + TextRenderer<Font = Font>
    + advanced_svg::Renderer
    + advanced_image::Renderer<Handle = advanced_image::Handle>
{
}

/// Routes a widget into the compositor's post-glass foreground layer.
pub fn glass_foreground<'a, Message, R>(
    content: impl Into<Element<'a, Message, Theme, R>>,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: CoreRenderer + GlassForegroundRenderer + 'a,
{
    Element::new(GlassForeground::new(content.into()))
}

/// Routes a widget into the compositor's topmost overlay layer.
pub fn glass_overlay<'a, Message, R>(
    content: impl Into<Element<'a, Message, Theme, R>>,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: CoreRenderer + GlassForegroundRenderer + 'a,
{
    Element::new(GlassOverlay::new(content.into()))
}

impl<T> ComponentRenderer for T where
    T: CoreRenderer
        + TextRenderer<Font = Font>
        + advanced_svg::Renderer
        + advanced_image::Renderer<Handle = advanced_image::Handle>
{
}

fn palette(theme: &Theme) -> UiPalette {
    UiTheme::from_iced(theme).palette()
}

// ---------------------------------------------------------------------------
// Text styles
// ---------------------------------------------------------------------------

/// Secondary (de-emphasized) text color.
pub fn secondary_text(theme: &Theme) -> text::Style {
    text::Style { color: Some(palette(theme).text_secondary) }
}

/// Tertiary text color, used for disclosure chevrons and placeholders.
pub fn tertiary_text(theme: &Theme) -> text::Style {
    text::Style { color: Some(palette(theme).text_tertiary) }
}

// ---------------------------------------------------------------------------
// Container and surface styles
// ---------------------------------------------------------------------------

/// The opaque content area behind the settings rows.
pub fn content_surface(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(palette.content_background)),
        ..container::Style::default()
    }
}

/// A transparent content surface for a compositor-owned background.
///
/// The demo seeds its opaque right-hand panel before Iced draws the source
/// scene. Keeping this parent transparent lets the source and glass geometry
/// remain separate without adding another opaque rectangle.
pub fn compositor_content_surface(_theme: &Theme) -> container::Style {
    container::Style::default()
}

/// The window's opaque titlebar strip on platforms that keep one.
pub fn titlebar_surface(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(palette.window_background)),
        ..container::Style::default()
    }
}

/// A transparent parent surface used when the compositor owns the backdrop.
pub fn transparent_surface(_theme: &Theme) -> container::Style {
    container::Style::default()
}

/// The transparent sidebar; the compositor owns its glass background.
pub fn sidebar_surface(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(palette(theme).text_primary),
        ..container::Style::default()
    }
}

/// A rounded settings group with a hairline border.
pub fn group_surface(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        text_color: Some(palette.text_primary),
        background: Some(Background::Color(palette.group_background)),
        border: Border::default()
            .rounded(UiCornerStyle::GROUP.radius())
            .width(1.0)
            .color(palette.group_border),
        ..container::Style::default()
    }
}

/// The hairline separating rows inside a settings group.
pub fn group_separator(theme: &Theme) -> rule::Style {
    rule::Style {
        color: palette(theme).separator,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

// ---------------------------------------------------------------------------
// Control styles (macOS-aligned)
// ---------------------------------------------------------------------------

/// macOS switch: accent track when on, gray track when off, white knob.
pub fn toggler_style(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let palette = palette(theme);
    let is_toggled = match status {
        toggler::Status::Active { is_toggled }
        | toggler::Status::Hovered { is_toggled }
        | toggler::Status::Disabled { is_toggled } => is_toggled,
    };
    toggler::Style {
        background: Background::Color(if is_toggled {
            palette.accent
        } else {
            palette.control_track_off
        }),
        background_border_width: 0.0,
        background_border_color: Color::TRANSPARENT,
        foreground: Background::Color(Color::WHITE),
        foreground_border_width: 0.5,
        foreground_border_color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
        text_color: None,
        border_radius: None,
        padding_ratio: 0.10,
    }
}

/// macOS slider: accent-filled rail, white circular handle.
pub fn slider_style(theme: &Theme, _status: slider::Status) -> slider::Style {
    let palette = palette(theme);
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(palette.accent),
                Background::Color(palette.control_track_off),
            ),
            width: 4.0,
            border: Border::default(),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: Background::Color(Color::WHITE),
            border_width: 0.5,
            border_color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
        },
    }
}

/// macOS popup button used by pick lists.
pub fn pick_list_style(theme: &Theme, _status: pick_list::Status) -> pick_list::Style {
    let palette = palette(theme);
    pick_list::Style {
        text_color: palette.text_primary,
        placeholder_color: palette.text_tertiary,
        handle_color: palette.text_secondary,
        background: Background::Color(palette.group_background),
        border: Border::default()
            .rounded(UiCornerStyle::PICKER.radius())
            .width(1.0)
            .color(palette.group_border),
    }
}

/// The floating menu opened by a pick list.
pub fn menu_style(theme: &Theme) -> iced::overlay::menu::Style {
    let palette = palette(theme);
    iced::overlay::menu::Style {
        background: Background::Color(Color { a: 1.0, ..palette.group_background }),
        border: Border::default()
            .rounded(UiCornerStyle::MENU.radius())
            .width(1.0)
            .color(palette.group_border),
        text_color: palette.text_primary,
        selected_text_color: Color::WHITE,
        selected_background: Background::Color(palette.accent),
        shadow: Shadow { color: palette.shadow, offset: Vector::new(0.0, 4.0), blur_radius: 16.0 },
    }
}

/// macOS overlay scrollbars: a thin, rounded, translucent scroller.
pub fn scrollable_style(theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    let palette = palette(theme);
    let scroller = scrollable::Scroller {
        background: Background::Color(Color { a: 0.35, ..palette.text_secondary }),
        border: Border::default(),
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail { background: None, border: Border::default(), scroller },
        horizontal_rail: scrollable::Rail { background: None, border: Border::default(), scroller },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(palette.content_background),
            border: Border::default().rounded(UiCornerStyle::MENU.radius()),
            shadow: Shadow::default(),
            icon: palette.text_secondary,
        },
    }
}

/// A scrollable list that lets the compositor-owned sidebar surface remain
/// visible instead of painting the content pane's background into the rail.
pub fn sidebar_scrollable_style(theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    let palette = palette(theme);
    let scroller = scrollable::Scroller {
        background: Background::Color(Color { a: 0.35, ..palette.text_secondary }),
        border: Border::default(),
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail { background: None, border: Border::default(), scroller },
        horizontal_rail: scrollable::Rail { background: None, border: Border::default(), scroller },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(Color::TRANSPARENT),
            border: Border::default().rounded(UiCornerStyle::SCROLLBAR.radius()),
            shadow: Shadow::default(),
            icon: palette.text_secondary,
        },
    }
}

/// macOS progress bar: accent fill on a gray track.
pub fn progress_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).group_background)),
        ..container::Style::default()
    }
}

// ---------------------------------------------------------------------------
// Building blocks
// ---------------------------------------------------------------------------

/// A tinted vector icon. Prefer [`icon_tinted`] for theme-driven colors.
pub fn icon<'a, Message, R>(icon: UiIcon, size: f32, color: Color) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: advanced_svg::Renderer + advanced_image::Renderer<Handle = advanced_image::Handle> + 'a,
{
    ui_icon(icon, size, color)
}

/// A vector icon whose tint is resolved from the active [`UiPalette`].
pub fn icon_tinted<'a, Message, R>(
    icon: UiIcon,
    size: f32,
    tint: impl Fn(&UiPalette) -> Color + 'a,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: advanced_svg::Renderer + 'a,
{
    iced::widget::svg(svg_handle(icon))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |theme, _status| iced::widget::svg::Style {
            color: Some(tint(&palette(theme))),
        })
        .into()
}

/// A vector icon with an explicit compositing opacity.
///
/// Iced's SVG colour filter uses the RGB channels as a rasterization key and
/// does not apply the alpha channel of the tint to the resulting image. Use
/// this helper for animated fades so opacity is sent through the SVG widget's
/// compositing path instead of disappearing only when the widget is removed.
pub fn icon_tinted_with_opacity<'a, Message, R>(
    icon: UiIcon,
    size: f32,
    color: Color,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: advanced_svg::Renderer + 'a,
{
    let opacity = color.a.clamp(0.0, 1.0);
    let color = Color { a: 1.0, ..color };
    iced::widget::svg(svg_handle(icon))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .opacity(opacity)
        .style(move |_theme, _status| iced::widget::svg::Style { color: Some(color) })
        .into()
}

fn svg_handle(icon: UiIcon) -> iced::widget::svg::Handle {
    let UiIconAsset::Svg(source) = icon.asset() else {
        unreachable!("icon_tinted is only used with vector icons")
    };
    iced::widget::svg::Handle::from_memory(source.as_bytes())
}

/// A colored rounded-square chip with a white symbol, as used by the macOS
/// settings sidebar. Raster icons (which already ship their own colored
/// artwork) render plain, without a chip.
pub fn icon_chip<'a, Message, R>(
    icon: UiIcon,
    chip_color: Color,
    size: f32,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: advanced_svg::Renderer + advanced_image::Renderer<Handle = advanced_image::Handle> + 'a,
{
    if matches!(icon.asset(), UiIconAsset::Png(_)) {
        return ui_icon(icon, size, Color::WHITE);
    }
    container(ui_icon(icon, size * 0.58, Color::WHITE))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fixed(size))
        .center_y(Length::Fixed(size))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(chip_color)),
            border: Border::default()
                .rounded(UiCornerStyle::ICON_CHIP.with_radius(size * 0.26).radius()),
            ..container::Style::default()
        })
        .into()
}

/// A large in-page section title with a subtitle.
pub fn section_heading<'a, Message, R>(
    title: &'a str,
    subtitle: &'a str,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: TextRenderer<Font = Font> + 'a,
{
    column![
        text(title).size(font::size::TITLE).font(font::ui_font(iced::font::Weight::Semibold)),
        text(subtitle).size(font::size::CAPTION).style(secondary_text),
    ]
    .spacing(3)
    .padding(Padding::new(8.0).left(4.0))
    .into()
}

/// A rounded group of settings rows separated by inset hairlines.
pub fn settings_group<'a, Message, R>(
    rows: Vec<Element<'a, Message, Theme, R>>,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: CoreRenderer + 'a,
{
    let mut entries: Vec<Element<'a, Message, Theme, R>> = Vec::with_capacity(rows.len() * 2);
    for (index, entry) in rows.into_iter().enumerate() {
        if index > 0 {
            entries.push(
                container(rule::horizontal(1).style(group_separator))
                    .padding(Padding::new(0.0).left(16.0))
                    .into(),
            );
        }
        entries.push(entry);
    }
    container(column(entries).width(Length::Fill)).width(Length::Fill).style(group_surface).into()
}

/// A settings row: label (and optional detail text) on the left, any control
/// on the right.
pub fn setting_row<'a, Message, R>(
    label: impl Into<String>,
    detail: Option<String>,
    control: impl Into<Element<'a, Message, Theme, R>>,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: TextRenderer<Font = Font> + 'a,
{
    let mut texts =
        column![text(label.into()).size(font::size::BODY)].spacing(3).width(Length::Fill);
    if let Some(detail) = detail {
        texts = texts.push(text(detail).size(font::size::CAPTION).style(secondary_text));
    }
    container(row![texts, control.into()].spacing(18).align_y(Alignment::Center))
        .width(Length::Fill)
        .padding([12, 14])
        .into()
}

/// A whole-row link with a trailing disclosure chevron.
pub fn setting_link<'a, Message, R>(
    label: impl Into<String>,
    detail: Option<String>,
    on_press: Message,
) -> Element<'a, Message, Theme, R>
where
    Message: Clone + 'a,
    R: ComponentRenderer + 'a,
{
    let chevron = icon_tinted(UiIcon::ChevronRight, 14.0, |palette| palette.text_tertiary);
    let row_content = setting_row(label, detail, chevron);
    button(row_content)
        .on_press(on_press)
        .width(Length::Fill)
        .padding(0)
        .style(link_row_style)
        .into()
}

/// A whole-row link with a leading neutral icon chip, as used for the
/// top-level entries of a settings section ("About This Mac", ...).
pub fn setting_link_with_icon<'a, Message, R>(
    icon: UiIcon,
    label: impl Into<String>,
    detail: Option<String>,
    on_press: Message,
) -> Element<'a, Message, Theme, R>
where
    Message: Clone + 'a,
    R: ComponentRenderer + 'a,
{
    let chip = icon_chip(icon, Color::from_rgb(0.55, 0.55, 0.57), 20.0);
    let chevron = icon_tinted(UiIcon::ChevronRight, 14.0, |palette| palette.text_tertiary);
    let mut texts =
        column![text(label.into()).size(font::size::BODY)].spacing(3).width(Length::Fill);
    if let Some(detail) = detail {
        texts = texts.push(text(detail).size(font::size::CAPTION).style(secondary_text));
    }
    let row_content = container(row![chip, texts, chevron].spacing(12).align_y(Alignment::Center))
        .width(Length::Fill)
        .padding([12, 14]);
    button(row_content)
        .on_press(on_press)
        .width(Length::Fill)
        .padding(0)
        .style(link_row_style)
        .into()
}

/// A settings row with a macOS-style toggle switch.
pub fn setting_toggle<'a, Message, R>(
    label: impl Into<String>,
    detail: Option<String>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: TextRenderer<Font = Font> + 'a,
{
    let switch = toggler(value).on_toggle(on_toggle).size(22.0).style(toggler_style);
    setting_row(label, detail, switch)
}

/// A settings row with a macOS-style slider and a trailing value label.
pub fn setting_slider<'a, Message, R>(
    label: impl Into<String>,
    value_text: impl Into<String>,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message, Theme, R>
where
    Message: Clone + 'a,
    R: TextRenderer<Font = Font> + 'a,
{
    setting_slider_with_step(label, value_text, value, range, 1.0, on_change)
}

/// A settings-style slider with an explicit floating-point step.
///
/// Iced's default slider step is `1`, which is appropriate for integer-like
/// settings but makes normalized `0.0..=1.0` material controls effectively
/// binary. This variant lets visual tuning panels choose a useful resolution.
pub fn setting_slider_with_step<'a, Message, R>(
    label: impl Into<String>,
    value_text: impl Into<String>,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    step: f32,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message, Theme, R>
where
    Message: Clone + 'a,
    R: TextRenderer<Font = Font> + 'a,
{
    let control = row![
        text(value_text.into()).size(font::size::CAPTION).style(secondary_text),
        slider(range, value, on_change).step(step).width(Length::Fixed(170.0)).style(slider_style),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    setting_row(label, None, control)
}

/// A settings row with a popup pick list.
pub fn setting_pick_list<'a, T, Message, R>(
    label: impl Into<String>,
    detail: Option<String>,
    options: Vec<T>,
    selected: Option<T>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message, Theme, R>
where
    T: ToString + PartialEq + Clone + 'a,
    Message: Clone + 'a,
    R: TextRenderer<Font = Font> + 'a,
{
    let picker =
        pick_list(options, selected, on_selected).style(pick_list_style).menu_style(menu_style);
    setting_row(label, detail, picker)
}

/// A sidebar row: colored icon chip, label, and an accent selection pill.
pub fn sidebar_item<'a, Message, R>(
    icon: UiIcon,
    chip_color: Color,
    label: &'a str,
    selected: bool,
    on_press: Message,
) -> Element<'a, Message, Theme, R>
where
    Message: Clone + 'a,
    R: ComponentRenderer + 'a,
{
    let label_style = move |theme: &Theme| text::Style {
        color: Some(if selected { Color::WHITE } else { palette(theme).text_primary }),
    };
    button(
        row![
            icon_chip(icon, chip_color, 22.0),
            text(label)
                .size(font::size::BODY)
                .font(font::ui_font(iced::font::Weight::Semibold))
                .style(label_style),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .on_press(on_press)
    .width(Length::Fill)
    .padding([4, 6])
    .style(move |theme, status| sidebar_item_style(theme, status, selected))
    .into()
}

/// Groups a list of sidebar items into a vertical group with macOS standard 2px spacing,
/// routing each item into the compositor foreground layer.
pub fn sidebar_group<'a, Message, R>(
    items: impl IntoIterator<Item = Element<'a, Message, Theme, R>>,
) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: CoreRenderer + GlassForegroundRenderer + 'a,
{
    container(column(items.into_iter().map(glass_foreground).collect::<Vec<_>>()).spacing(2))
        .width(Length::Fill)
        .into()
}

/// Standard spacing gap between sidebar groups.
pub fn sidebar_gap<'a, Message, R>() -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: CoreRenderer + 'a,
{
    space().height(Length::Fixed(8.0)).into()
}

/// A standard sidebar section header label.
pub fn sidebar_section_header<'a, Message, R>(title: &'a str) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: ComponentRenderer + 'a,
{
    text(title)
        .size(font::size::CAPTION)
        .font(font::ui_font(iced::font::Weight::Bold))
        .style(|theme: &Theme| text::Style { color: Some(palette(theme).text_secondary) })
        .into()
}

/// The transparent, glass-backed search field for a sidebar.
pub fn search_field<'a, Message, R>(
    id: GlassId,
    bounds: Rect,
    scheme: UiColorScheme,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message, Theme, R>
where
    Message: Clone + 'static,
    R: ComponentRenderer + GlassForegroundRenderer + 'static,
{
    let field = row![
        icon_tinted(UiIcon::Search, 13.0, |palette| palette.text_secondary),
        text_input("Search", value)
            .on_input(on_input)
            .width(Length::Fill)
            .size(font::size::BODY)
            .padding([4, 0])
            .style(search_input_style),
    ]
    .spacing(7)
    .align_y(Alignment::Center);
    glass_surface(
        id,
        bounds,
        GlassRole::SearchField,
        scheme,
        Padding::new(2.0).left(10.0).right(10.0),
        None,
        field,
    )
}

/// A transparent foreground laid over a compositor glass surface.
///
/// The glass itself is drawn by the renderer from the [`GlassContainer`]
/// scene node; the foreground hosts the interactive widgets.
pub fn glass_surface<'a, Message, R>(
    id: GlassId,
    bounds: Rect,
    role: GlassRole,
    scheme: UiColorScheme,
    padding: Padding,
    foreground_alpha: Option<f32>,
    content: impl Into<Element<'a, Message, Theme, R>>,
) -> Element<'a, Message, Theme, R>
where
    Message: 'static,
    R: CoreRenderer + GlassForegroundRenderer + 'static,
{
    let theme = UiTheme::new(scheme);
    // This node is a compositor surface. Its material fill must not be
    // painted again by the Iced overlay, otherwise the shader's refracted
    // result is covered by a visible white/color-tinted block.
    let mut overlay_material = GlassMaterial::clear();
    overlay_material.tint = GlassColor::transparent();
    let background = GlassContainer::new(id, bounds)
        .shape(theme.glass_shape(role))
        .material(overlay_material)
        // The compositor owns the full visual surface, including its edge
        // light and shadow. Keep the Iced fallback chrome transparent so it
        // cannot duplicate or clip the expanded GPU effect region.
        .chrome(theme.compositor_chrome(role))
        .into_element::<Message, Theme, R>();
    let content = container(content.into())
        .width(Length::Fixed(bounds.width))
        .height(Length::Fixed(bounds.height))
        .padding(padding)
        .align_y(Alignment::Center)
        .style(move |iced_theme| {
            let background = foreground_alpha.map(|alpha| {
                Background::Color(palette(iced_theme).content_background.scale_alpha(alpha))
            });
            container::Style { background, ..container::Style::default() }
        });
    // Toolbar copy is part of the final chrome layer. The fused titlebar can
    // receive a second, directional blur pass, so its title and controls
    // must be composited after every material pass rather than left in the
    // intermediate foreground surface.
    let foreground: Element<'a, Message, Theme, R> = glass_overlay(content);
    iced::widget::stack![background, foreground].into()
}

/// A small rounded accent-color dot.
pub fn accent_dot<Message, R>(color: Color) -> Element<'static, Message, Theme, R>
where
    Message: 'static,
    R: CoreRenderer + 'static,
{
    container(space())
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(move |theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border::default()
                .rounded(UiCornerStyle::CONTROL.with_radius(7.0).radius())
                .width(1.0)
                .color(palette(theme).group_border),
            ..container::Style::default()
        })
        .into()
}

/// The accent-filled progress bar used at the bottom of settings pages.
pub fn progress_indicator<'a, Message, R>(value: f32) -> Element<'a, Message, Theme, R>
where
    Message: 'a,
    R: CoreRenderer + 'a,
{
    container(progress_bar(0.0..=100.0, value))
        .width(Length::Fill)
        .height(Length::Fixed(4.0))
        .style(progress_style)
        .into()
}

// ---------------------------------------------------------------------------
// Button styles
// ---------------------------------------------------------------------------

/// A transparent whole-row button that highlights on hover/press.
pub fn link_row_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);
    let background = match status {
        button::Status::Hovered => palette.hover,
        button::Status::Pressed => palette.sidebar_selection,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.text_primary,
        ..button::Style::default()
    }
}

/// The sidebar selection pill: accent fill with white label, matching macOS.
pub fn sidebar_item_style(theme: &Theme, status: button::Status, selected: bool) -> button::Style {
    let palette = palette(theme);
    let accent_pressed = Color { a: 1.0, ..palette.accent };
    let (background, text_color) = match (selected, status) {
        (true, button::Status::Pressed) => (accent_pressed.scale_alpha(0.85), Color::WHITE),
        (true, _) => (accent_pressed, Color::WHITE),
        (false, button::Status::Hovered | button::Status::Pressed) => {
            (palette.hover, palette.text_primary)
        }
        (false, _) => (Color::TRANSPARENT, palette.text_primary),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border::default().rounded(UiCornerStyle::PICKER.radius()),
        ..button::Style::default()
    }
}

/// A segmented button in a small control cluster (Auto / Light / Dark).
pub fn segmented_style(theme: &Theme, status: button::Status, selected: bool) -> button::Style {
    let palette = palette(theme);
    let background = if selected {
        palette.selection
    } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        palette.hover
    } else {
        Color::TRANSPARENT
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.text_primary,
        border: Border::default()
            .rounded(UiCornerStyle::CONTROL.with_radius(7.0).radius())
            .width(1.0)
            .color(Color::from_rgba(
                palette.group_border.r,
                palette.group_border.g,
                palette.group_border.b,
                if selected { 0.30 } else { palette.group_border.a },
            )),
        ..button::Style::default()
    }
}

/// A transparent toolbar button that highlights on hover/press.
pub fn toolbar_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);
    let background = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        palette.hover
    } else {
        Color::TRANSPARENT
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.text_primary,
        border: Border::default().rounded(UiCornerStyle::GROUP.radius()),
        ..button::Style::default()
    }
}

/// The transparent text input living inside a glass search field.
pub fn search_input_style(theme: &Theme, _status: text_input::Status) -> text_input::Style {
    let palette = palette(theme);
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default()
            .rounded(UiCornerStyle::CONTROL.radius())
            .width(0.0)
            .color(Color::TRANSPARENT),
        icon: palette.text_secondary,
        placeholder: palette.text_secondary,
        value: palette.text_primary,
        selection: palette.selection,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggler_track_follows_state() {
        let theme = Theme::Dark;
        let on = toggler_style(&theme, toggler::Status::Active { is_toggled: true });
        let off = toggler_style(&theme, toggler::Status::Active { is_toggled: false });
        assert_ne!(on.background, off.background);
    }

    #[test]
    fn sidebar_selection_differs_from_list_selection() {
        let palette = UiTheme::dark().palette();
        assert_ne!(palette.sidebar_selection, palette.selection);
    }
}
