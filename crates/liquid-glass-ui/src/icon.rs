//! System icons extracted from the operating system (SF Symbols and bespoke
//! bundle assets on macOS).
//!
//! Assets are authored once with the `extract_sf_symbols` developer binary in
//! the playground example and committed under `assets/icons/`. Vector icons
//! are monochrome paths recolored through iced's svg style color filter;
//! raster assets keep their baked-in colors.

use std::{cell::RefCell, collections::HashMap};

use iced::{
    Color, Element, Length, Theme,
    advanced::{image as advanced_image, svg as advanced_svg},
    widget::{image, svg},
};

use iced::advanced::image::Handle as ImageHandle;

thread_local! {
    /// `iced::image::Handle::from_bytes` deliberately assigns a fresh ID.
    /// Recreating it from an embedded PNG during every declarative `view()`
    /// update defeats the GPU atlas cache and eventually blocks its worker
    /// queue. Retain one handle per icon on the UI thread instead.
    static RASTER_HANDLES: RefCell<HashMap<UiIcon, ImageHandle>> = RefCell::new(HashMap::new());
}

/// A system icon available to UI components.
///
/// Most icons are SF Symbols extracted as vector SVG. A few are bespoke
/// colored assets (`Bluetooth`, `FileVault`, ...) that Apple ships only as raster
/// images inside feature bundles; those are extracted as PNG and never tinted.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiIcon {
    Gear,
    /// The Appearance tile glyph; automatically swaps between its light and
    /// dark renditions with the active color scheme, like the real Settings
    /// sidebar does.
    Appearance,
    /// The dark-scheme rendition of [`UiIcon::Appearance`]; rarely needed
    /// directly since `Appearance` self-themes.
    AppearanceDark,
    Bell,
    Privacy,
    Search,
    ChevronLeft,
    ChevronRight,
    WindowClose,
    WindowMinimize,
    WindowMaximize,
    WindowZoom,
    Question,
    Info,
    Laptop,
    SoftwareUpdate,
    Keyboard,
    Mouse,
    Sound,
    Wifi,
    Bluetooth,
    SoftwareUpdateBadge,
    FileVault,
    Notifications,
    // System Settings sidebar assets. These are kept separate from the older
    // demo glyphs above because the sidebar assets include Apple's baked-in
    // color treatment where available.
    SystemAppleAccount,
    SystemFamilySharing,
    SystemWifi,
    SystemBluetooth,
    SystemNetwork,
    SystemVpn,
    SystemBattery,
    SystemGeneral,
    SystemAccessibility,
    SystemMenuBar,
    SystemSpotlight,
    SystemWallpaper,
    SystemAppearance,
    SystemDisplays,
    SystemDock,
    SystemSiri,
    SystemNotifications,
    SystemSound,
    SystemFocus,
    SystemScreenTime,
    SystemLockScreen,
    SystemPrivacySecurity,
    SystemTouchId,
    SystemUsersGroups,
    SystemInternetAccounts,
    SystemWallet,
    SystemGameCenter,
    SystemICloud,
    SystemAirPods,
    SystemKeyboard,
    SystemTrackpad,
    SystemGameController,
    SystemPrintersScanners,
}

/// The embedded bytes behind a [`UiIcon`].
#[derive(Clone, Copy, Debug)]
pub enum UiIconAsset {
    /// A monochrome vector path; recolor via iced's svg style color filter.
    Svg(&'static str),
    /// A colored raster asset extracted from a system feature bundle.
    Png(&'static [u8]),
}

impl UiIcon {
    /// The embedded asset backing this icon.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn asset(self) -> UiIconAsset {
        match self {
            Self::Gear => UiIconAsset::Svg(include_str!("../assets/icons/gear.svg")),
            Self::Appearance => UiIconAsset::Svg(include_str!("../assets/icons/appearance.svg")),
            Self::AppearanceDark => {
                UiIconAsset::Svg(include_str!("../assets/icons/appearance_dark.svg"))
            }
            Self::Bell => UiIconAsset::Svg(include_str!("../assets/icons/bell.svg")),
            Self::Privacy => UiIconAsset::Svg(include_str!("../assets/icons/privacy.svg")),
            Self::Search => UiIconAsset::Svg(include_str!("../assets/icons/search.svg")),
            Self::ChevronLeft => UiIconAsset::Svg(include_str!("../assets/icons/chevron_left.svg")),
            Self::ChevronRight => {
                UiIconAsset::Svg(include_str!("../assets/icons/chevron_right.svg"))
            }
            Self::WindowClose => UiIconAsset::Svg(include_str!("../assets/icons/window_close.svg")),
            Self::WindowMinimize => {
                UiIconAsset::Svg(include_str!("../assets/icons/window_minimize.svg"))
            }
            Self::WindowMaximize => {
                UiIconAsset::Svg(include_str!("../assets/icons/window_maximize.svg"))
            }
            Self::WindowZoom => UiIconAsset::Svg(include_str!("../assets/icons/window_zoom.svg")),
            Self::Question => UiIconAsset::Svg(include_str!("../assets/icons/question.svg")),
            Self::Info => UiIconAsset::Svg(include_str!("../assets/icons/info.svg")),
            Self::Laptop => UiIconAsset::Svg(include_str!("../assets/icons/laptop.svg")),
            Self::SoftwareUpdate => {
                UiIconAsset::Svg(include_str!("../assets/icons/software_update.svg"))
            }
            Self::Keyboard => UiIconAsset::Svg(include_str!("../assets/icons/keyboard.svg")),
            Self::Mouse => UiIconAsset::Svg(include_str!("../assets/icons/mouse.svg")),
            Self::Sound => UiIconAsset::Svg(include_str!("../assets/icons/sound.svg")),
            Self::Wifi => UiIconAsset::Svg(include_str!("../assets/icons/wifi.svg")),
            Self::Bluetooth => UiIconAsset::Png(include_bytes!("../assets/icons/bluetooth.png")),
            Self::SoftwareUpdateBadge => {
                UiIconAsset::Png(include_bytes!("../assets/icons/software_update_badge.png"))
            }
            Self::FileVault => UiIconAsset::Png(include_bytes!("../assets/icons/filevault.png")),
            Self::Notifications => {
                UiIconAsset::Png(include_bytes!("../assets/icons/notifications.png"))
            }
            Self::SystemAppleAccount => {
                UiIconAsset::Svg(include_str!("../assets/system-settings/01-apple-account.svg"))
            }
            Self::SystemFamilySharing => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/02-family-sharing.png"))
            }
            Self::SystemWifi => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/03-wifi.png"))
            }
            Self::SystemBluetooth => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/04-bluetooth.png"))
            }
            Self::SystemNetwork => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/05-network.png"))
            }
            Self::SystemVpn => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/06-vpn.png"))
            }
            Self::SystemBattery => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/07-battery.png"))
            }
            Self::SystemGeneral => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/08-general.png"))
            }
            Self::SystemAccessibility => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/09-accessibility.png"))
            }
            Self::SystemMenuBar => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/10-menu-bar.png"))
            }
            Self::SystemSpotlight => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/11-spotlight.png"))
            }
            Self::SystemWallpaper => {
                UiIconAsset::Svg(include_str!("../assets/system-settings/12-wallpaper.svg"))
            }
            Self::SystemAppearance => {
                UiIconAsset::Svg(include_str!("../assets/system-settings/13-appearance.svg"))
            }
            Self::SystemDisplays => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/14-displays.png"))
            }
            Self::SystemDock => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/15-dock.png"))
            }
            Self::SystemSiri => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/16-siri.png"))
            }
            Self::SystemNotifications => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/17-notifications.png"))
            }
            Self::SystemSound => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/18-sound.png"))
            }
            Self::SystemFocus => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/19-focus.png"))
            }
            Self::SystemScreenTime => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/20-screen-time.png"))
            }
            Self::SystemLockScreen => {
                UiIconAsset::Svg(include_str!("../assets/system-settings/21-lock-screen.svg"))
            }
            Self::SystemPrivacySecurity => UiIconAsset::Png(include_bytes!(
                "../assets/system-settings/22-privacy-security.png"
            )),
            Self::SystemTouchId => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/23-touch-id.png"))
            }
            Self::SystemUsersGroups => {
                UiIconAsset::Svg(include_str!("../assets/system-settings/24-users-groups.svg"))
            }
            Self::SystemInternetAccounts => UiIconAsset::Png(include_bytes!(
                "../assets/system-settings/25-internet-accounts.png"
            )),
            Self::SystemWallet => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/26-wallet.png"))
            }
            Self::SystemGameCenter => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/27-game-center.png"))
            }
            Self::SystemICloud => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/28-icloud.png"))
            }
            Self::SystemAirPods => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/29-airpods.png"))
            }
            Self::SystemKeyboard => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/30-keyboard.png"))
            }
            Self::SystemTrackpad => {
                UiIconAsset::Png(include_bytes!("../assets/system-settings/31-trackpad.png"))
            }
            Self::SystemGameController => {
                UiIconAsset::Svg(include_str!("../assets/system-settings/32-game-controller.svg"))
            }
            Self::SystemPrintersScanners => {
                UiIconAsset::Svg(include_str!("../assets/system-settings/33-printers-scanners.svg"))
            }
        }
    }
}

fn raster_handle(icon: UiIcon, bytes: &'static [u8]) -> ImageHandle {
    RASTER_HANDLES.with_borrow_mut(|handles| {
        handles.entry(icon).or_insert_with(|| ImageHandle::from_bytes(bytes)).clone()
    })
}

/// Renders a [`UiIcon`] as an `Element` laid out at `size` points square.
///
/// Vector icons are tinted to `color`; raster assets keep their baked-in
/// colors. `Appearance` overlays its two renditions and fades between them
/// via the svg color filter, following the theme's color scheme.
#[must_use]
pub fn icon<'a, Message, Renderer>(
    icon: UiIcon,
    size: f32,
    color: Color,
) -> Element<'a, Message, Theme, Renderer>
where
    Renderer: advanced_svg::Renderer + advanced_image::Renderer<Handle = ImageHandle> + 'a,
    Message: 'a,
{
    if icon == UiIcon::Appearance {
        let layer = |variant: UiIcon, visible_in_dark: bool| {
            svg(svg::Handle::from_memory(match variant.asset() {
                UiIconAsset::Svg(source) => source.as_bytes(),
                UiIconAsset::Png(_) => unreachable!("Appearance is a vector icon"),
            }))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .style(move |theme: &Theme, _status| {
                let visible = theme.extended_palette().is_dark == visible_in_dark;
                svg::Style { color: Some(if visible { color } else { Color::TRANSPARENT }) }
            })
        };
        return iced::widget::stack![
            layer(UiIcon::Appearance, false),
            layer(UiIcon::AppearanceDark, true),
        ]
        .into();
    }
    match icon.asset() {
        UiIconAsset::Svg(source) => svg(svg::Handle::from_memory(source.as_bytes()))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .style(move |_theme, _status| svg::Style { color: Some(color) })
            .into(),
        UiIconAsset::Png(bytes) => image(raster_handle(icon, bytes))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_icons_contain_only_paths() {
        for icon in [
            UiIcon::Gear,
            UiIcon::Appearance,
            UiIcon::AppearanceDark,
            UiIcon::Bell,
            UiIcon::Privacy,
            UiIcon::Search,
            UiIcon::ChevronLeft,
            UiIcon::ChevronRight,
            UiIcon::WindowClose,
            UiIcon::WindowMinimize,
            UiIcon::WindowZoom,
            UiIcon::Question,
            UiIcon::Info,
            UiIcon::Laptop,
            UiIcon::SoftwareUpdate,
            UiIcon::Keyboard,
            UiIcon::Mouse,
            UiIcon::Sound,
            UiIcon::Wifi,
            UiIcon::SystemAppleAccount,
            UiIcon::SystemWallpaper,
            UiIcon::SystemAppearance,
            UiIcon::SystemLockScreen,
            UiIcon::SystemUsersGroups,
            UiIcon::SystemGameController,
            UiIcon::SystemPrintersScanners,
        ] {
            let UiIconAsset::Svg(source) = icon.asset() else {
                panic!("{icon:?} should be a vector icon");
            };
            assert!(source.contains("<path"), "{icon:?} has no vector path");
            assert!(!source.contains("<image"), "{icon:?} embeds raster data");
        }
    }

    #[test]
    fn raster_icons_are_png() {
        for icon in [
            UiIcon::Bluetooth,
            UiIcon::SoftwareUpdateBadge,
            UiIcon::FileVault,
            UiIcon::Notifications,
        ] {
            let UiIconAsset::Png(bytes) = icon.asset() else {
                panic!("{icon:?} should be a raster icon");
            };
            assert!(bytes.starts_with(b"\x89PNG"), "{icon:?} is not a PNG");
        }
    }

    #[test]
    fn raster_icons_reuse_a_stable_iced_handle() {
        let UiIconAsset::Png(bytes) = UiIcon::SystemGeneral.asset() else {
            panic!("SystemGeneral should be a raster icon");
        };

        let first = raster_handle(UiIcon::SystemGeneral, bytes);
        let second = raster_handle(UiIcon::SystemGeneral, bytes);

        assert_eq!(first.id(), second.id());
    }
}
