# bmol-glass-ui

The **shared Liquid Glass widget layer**: the Iced glass surfaces and controls that
both the window shell and an application need.

It exists because one component has two consumers. A rounded, frosted Liquid Glass
panel is the shell's context menu *and* an in-app panel; a titlebar surface is chrome
in the shell and a header in an app. Those are the same widget, so they belong in a
crate that sits *below* both rather than inside one of them.

## Layering

```
liquid-rs            mechanism: GlassScene / GlassMaterial / variant / geometry / GPU
   |
bmol-designs         family style: tokens, semantic colours, GlassRole -> material
   |                 profiles, ClarityPolicy. Toolkit-free by default; the Iced
   |                 adapters sit behind its optional `iced` feature.
   |
liquid-glass-ui      shared Iced glass components            <-- this repository
   |
   +-- bmol-window-shell   window shell   } siblings: neither depends on the other
   +-- bmol-iced           application    }
```

## What belongs here

Shared glass surfaces and controls: panel, group, sidebar and titlebar surfaces,
context menu, popover, scroll view, the icon and font helpers, the traffic-light
control, and the `GlassForeground` / `GlassOverlay` layer routing.

## What does not

| Concern | Home |
| --- | --- |
| Glass mechanism (materials, variants, geometry, GPU) | `liquid-rs` |
| Design tokens, semantic colours, `GlassRole` profiles, `ClarityPolicy` | `bmol-designs` |
| Platform and window semantics: native setup, chrome config and metrics, drag bar, rim, resizer | `bmol-window-shell` |
| Traffic-light *semantics*: interaction state machine, material tuning, `GlassScene` builder | `bmol-window-shell` |
| Iced-facing window orchestration (`IcedWindowPolicy`, `IcedWindowController`, `WindowCommand`, `WindowDragArea`) | `bmol-iced` |
| Application composition, such as the Dock | `bmol-iced` |

## The traffic-light control

The window controls are the clearest justification for this crate existing.

Their sphere material is drawn by the GPU Liquid Glass compositor, but the Apple
vector glyphs are drawn by Iced. Those are different render passes, so the glyphs
must be routed through the compositor's **overlay** layer explicitly. A caller that
skips that step gets three blank circles and no error — which is exactly the bug
that made this module move down here.

`window_control_group` therefore wraps itself in `GlassOverlay`, and a caller only
has to place the returned element on the glass. What stays in `bmol-window-shell` is
the part that is about the *window*, not about drawing: which control means which
window command, the hover/press state machine and its press-scale spring, the
material tuning, and the `GlassScene` that turns a frame of that state into GPU
nodes. The widget reports pointer facts (`TrafficLightsEvent`) and knows nothing
about windows.

## Guards

`scripts/check-layering.sh` fails the build if either of these breaks:

1. `bmol-window-shell` reappears in the normal dependency graph, which would put
   the shell *above* this crate again instead of beside it;
2. the graph resolves more than one `liquid-glass-scene` source, which would mean
   two incompatible `GlassMaterial` types that cannot be passed across.

Run it in CI next to the tests.

## The theme

`src/theme.rs` is the single conversion point to `bmol-designs`. The palette and
chrome values live there; only `UiPalette` is re-typed in Iced's colour, built once
by `UiPalette::from(bmol_designs::UiPalette)`, and `UiTheme` delegates every method.
Converting at each access site was measured and rejected: the palette is reached
through six `palette()` calls but its fields are read seventy-two times.

## Releases

`v0.1.6` is the current release. Depend on the tag rather than `branch = "main"`
when you want a reproducible build.

If your application builds on `iced_wgpu::Renderer` directly rather than on a
compositor renderer, also enable the `iced-wgpu` feature: the traffic-light
widget routes its glyphs above the glass pass unconditionally, so a renderer
with no compositor still has to satisfy that bound. `bmol-window-shell` and
`bmol-iced` enable it for you.
