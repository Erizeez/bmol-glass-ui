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
context menu, popover, scroll view, the icon and font helpers, and the
`GlassForeground` / `GlassOverlay` layer routing.

## What does not

| Concern | Home |
| --- | --- |
| Glass mechanism (materials, variants, geometry, GPU) | `liquid-rs` |
| Design tokens, semantic colours, `GlassRole` profiles, `ClarityPolicy` | `bmol-designs` |
| Platform and window semantics: native setup, chrome config and metrics, drag bar, rim, resizer, traffic lights | `bmol-window-shell` |
| Iced-facing window orchestration (`IcedWindowPolicy`, `IcedWindowController`, `WindowCommand`, `WindowDragArea`) | `bmol-iced` |
| Application composition, such as the Dock | `bmol-iced` |

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

`v0.1.4` is the first release. Depend on the tag rather than `branch = "main"` when
you want a reproducible build.
