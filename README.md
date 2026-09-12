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
      "siblings, not parent and child" property, and it is worth guarding with a
      `cargo tree` check in CI.
- [ ] `theme.rs` is still a fork of `bmol-designs::theme`. It should merge back with
      the iced-facing adapters (`UiTheme::from_iced`, `ClarityPolicy`) behind an
      optional `iced` feature. This step also has to move `bmol-designs` off
      `liquid-rs` tag `v0.1.3` onto `branch = "main"`: today the graph carries two
      copies of `liquid-glass-scene`, so a material from `bmol-designs` is not the
      same type as one from this crate and must not be mixed.
- [ ] `bmol-iced` still hosts the original copy of this crate and its `dock.rs`
      (application composition, which belongs at the app layer). Rewiring it to this
      crate is what finally deletes the old copy.
- [ ] The asset helpers `app_icon_png` and `load_system_wallpaper_rgba` were used
      only by `dock.rs`, which never lived here. When `dock.rs` moves to the app
      layer it should take them from `bmol-window-platform` / `bmol-window-native`
      rather than from `bmol-window-shell`.
