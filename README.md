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

This repository was created by moving the crate out of `bmol-iced` verbatim, so the
layering above is the target, not yet the state:

- [x] Crate moved out of `bmol-iced` and builds standalone here.
- [ ] `assets` helpers (`app_icon_png`, `load_system_wallpaper_rgba`) sink into
      `bmol-window-platform`, which removes the `bmol-window-shell` dependency below.
- [ ] `theme.rs` (a fork of `bmol-designs::theme`) merges back into `bmol-designs`,
      with the iced-facing adapters behind an optional feature.
- [ ] `window.rs` retires: the window semantics it re-exports stay in `bmol-window-shell`.
- [ ] `dock.rs` (application composition) moves to `bmol-iced`.

Until those land, `bmol-window-shell` is still a dependency of this crate and
`bmol-iced` still hosts the original copy of the crate.
