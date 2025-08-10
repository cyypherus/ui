# UI

A WIP declarative UI crate for building native applications with smooth animations & fluid layout. This crate handles windowing, layout, rendering, animations, and user interaction.

Built with [winit](https://github.com/rust-windowing/winit), [vello](https://github.com/linebender/vello), [backer](https://github.com/cyypherus/backer), and [lilt](https://github.com/cyypherus/lilt) for simple, beautiful apps.

> [!WARNING]
> **Limitations**:
>
> - No raster support (images/GIFs/video)
> - Single window only
> - No rotation
> - Incomplete scrolling
> - Limited effects (blur/shadow only for rects)
> - No accessibility
> - Unknown RTL support
> - Untested on platforms besides macOS
> - This library is functional but very experimental. API stability is not a goal at all at this stage & it is likely you will encounter bugs.

## Features

Leverages specialized crates:

- **Declarative API**: Code structure mirrors UI hierarchy
- **Flexible layout**: Constraint-based system powered by backer
- **Smooth animations**: Frame-rate independent transitions using lilt
- **GPU rendering**: Compute shader-based vector graphics using vello
- **Probably Cross-platform**: Might work on Windows, macOS, and Linux?

## Getting Started

### Define your state

Create a state struct with the data your UI needs to track & state for views as necessary. 
Certain views (like toggles) require bindings to state & will render based on that state.

```rust
#[derive(Clone, Default)]
struct AppState {
    text: TextState,
    toggle: ToggleState,
    count: i32,
}
```

### Build your layout

Use the declarative layout API to define your interface structure.

```rust
fn main() {
    App::start(AppState::default(), || {
        dynamic(|state: &mut AppState, _| {
            column_spaced(
                20.,
                vec![
                    text_field(id!(), binding!(AppState, text))
                        .font_size(40)
                        .finish(),
                    toggle(id!(), binding!(AppState, toggle)).finish(),
                    button(id!(), binding!(AppState, button))
                        .text_label(format!("Count: {}", state.count))
                        .on_click(|s, _| s.count += 1)
                        .finish(),
                ],
            )
        })
    });
}
```

## [Examples](examples/)

Examples demonstrate various UI patterns & can be run directly with `cargo run --example <name>`.

### idle-hue

A complete color picker application showcasing theming, complex layout, and state management.

### buttons

Interactive button examples with custom styling and click handlers.

### basic

Simple text field and toggle examples to get started.

### scroller

Scrollable content with gesture handling.
