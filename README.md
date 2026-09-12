# bmol-glass-ui

The **shared Liquid Glass widget layer**: the iced glass surfaces and controls that
both the window shell and an application need.

It exists because one component has two consumers. A rounded, frosted Liquid Glass
panel is the shell's context menu *and* an in-app panel; a titlebar surface is chrome
in the shell and a header in an app. Those are the same widget, so they belong in a
crate that sits *below* both — not inside one of them.

## Layering

```
liquid-rs               mechanism: GlassScene / GlassMaterial / variant / geometry / GPU
   |
bmol-designs            family style: tokens, semantic colours, GlassRole -> material profiles
   |
liquid-glass-ui         shared iced glass components   <-- this repository
   |
   +-- bmol-window-shell      window shell   } siblings: neither depends on the other
   +-- bmol-iced              application    }
```

## What belongs here

- Shared glass surfaces and controls: panel/group/sidebar/titlebar surfaces, context
  menu, popover, scroll view, icon and font helpers, and the
  `GlassForeground` / `GlassOverlay` layer routing.

## What does not

| Concern | Home |
| --- | --- |
| Glass mechanism (materials, variants, geometry, GPU) | `liquid-rs` |
| Design tokens, semantic colours, `GlassRole` -> `GlassMaterial` profiles | `bmol-designs` |
| Window semantics (chrome config/plan/metrics, native setup, drag bar, rim, resizer) | `bmol-window-shell` |
| Application composition (dock and similar) | `bmol-iced` |

## Migration status

The layering above is the target; this is how far it has got.

- [x] Crate moved out of `bmol-iced` and builds standalone here.
- [x] `window.rs` retired. It was only a facade over twelve window-semantics items
      (`WindowShellController`, `setup_native_window`, chrome plan and metrics, the
      drag bar, rim and resizer), which stay in `bmol-window-shell` where they belong.
      `DEFAULT_WINDOW_CORNER_RADIUS` is a `bmol-designs` token and can be taken from
      there.
- [x] Consequently this crate no longer depends on `bmol-window-shell` at all;
      `cargo tree` shows the shell absent from the whole graph. That is the
      "siblings, not parent and child" property.
- [x] `scripts/check-layering.sh` guards it: it fails if `bmol-window-shell`
      reappears in the normal graph, or if the graph carries more than one
      `liquid-glass-scene` source (which would mean two incompatible
      `GlassMaterial` types). Run it in CI next to the tests.
- [x] `bmol-designs` moved to liquid-rs `branch = "main"` and this crate follows
      it, so the graph resolves a single `liquid-glass-scene`.
- [ ] `theme.rs` is still a fork of `bmol-designs::theme`. It should merge back with
      the iced-facing adapters (`UiTheme::from_iced`, `ClarityPolicy`) behind an
      optional `iced` feature. This step also has to move `bmol-designs` off
      `liquid-rs` tag `v0.1.3` onto `branch = "main"`: today the graph carries two
      copies of `liquid-glass-scene`, so a material from `bmol-designs` is not the
      same type as one from this crate and must not be mixed.
- [ ] `bmol-iced` still hosts the original copy of this crate and its `dock.rs`
      (application composition, which belongs at the app layer). Rewiring it to this
      crate is what finally deletes the old copy.
- [x] The asset helpers `app_icon_png` and `load_system_wallpaper_rgba` need no
      move: both are already defined in `bmol-window-native`, the lowest layer, and
      `bmol-window-shell` only re-exports them. `dock.rs` (which never lived here)
      just has to import them from `bmol-window-native`. Nothing had to be sunk
      into `bmol-window-platform`.

### What the theme merge actually involves

Measured against `bmol-designs`, the two `theme.rs` files differ in **one
structural way**: the fork types `UiPalette`'s eighteen fields as `iced::Color`,
while `bmol-designs` types them as the scene `Color`. `GlassChrome` is scene-typed
in both, and the fork already writes its palette *values* in the scene type
(`GlassColor::rgba(..)`) and converts at the boundary in a single helper.

So under the "optional iced feature" plan the port is close to mechanical:

- lift the fork's values, `GlassRole::ContextMenu`, `impl UiColorScheme` and
  `impl GlassChrome` into `bmol-designs`, keeping the scene `Color` palette;
- put the iced adapters (`from_iced`, the colour conversion) behind a new optional
  `iced` feature;
- switch this crate to re-export `bmol_designs`' theme instead of its own, which is
  the part that needs the roughly eighteen palette-field accesses in
  `components.rs`, `context_menu.rs`, `popover.rs` and `scroll_view.rs` to convert.

Evidence that the fork is the authoritative copy: it carries six substantive
commits (device-measured context-menu calibration, 64 pt heavy-blur dark/light
modes, layered rendering work) against `bmol-designs`' two, and it has a
`GlassRole::ContextMenu` variant that `bmol-designs` lacks.
