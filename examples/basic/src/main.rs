use parley::FontWeight;
use ui::*;

#[derive(Debug, Clone, Default)]
struct State {
    text: TextState,
    toggle: ToggleState,
    slider: SliderState,
    button: ButtonState,
    dropdown: DropdownState,
}

fn main() {
    App::builder(
        State {
            text: TextState {
                text: "Bio-luminescenct moss carpets power floating gardens while crystal-infused mycelium networks whisper data through the canopy above"
                    .to_string(),
                editing: false,
            },
            toggle: ToggleState::default(),
            slider: SliderState::default(),
            button: ButtonState::default(),
            dropdown: DropdownState::default(),
        },
        || {
            dynamic(|_, _: &mut AppState<State>| {
                column_spaced(
                    20.,
                    vec![
                        text(
                            id!(),
                            "Mycelial Networks Harmonize with Quantum-Grown Algae Towers",
                        )
                        .font_weight(FontWeight::BOLD)
                        .font_size(30)
                        .wrap()
                        .finish(),
                        text_field(id!(), binding!(State, text)).wrap().highlight_fill(Color::from_rgb8(255, 0,0)).finish(),
                        toggle(id!(), binding!(State, toggle)).finish().height(50.),
                        slider(id!(), binding!(State, slider)).finish().height(50.),
                        dropdown(id!(), binding!(State, dropdown), vec![
                            text(id!(), "Luminescent Moss"),
                            text(id!(), "Crystal Mycelium"),
                            text(id!(), "Quantum Algae"),
                            text(id!(), "Floating Gardens"),
                            text(id!(), "Cerebral Forests"),
                            text(id!(), "Glass Marrow"),
                        ]).finish().height(50.),
                        button(id!(), binding!(State, button)).finish().height(50.),
                    ],
                )
                .pad(20.)
            })
        },
    )
    .inner_size(800, 600)
    // .resizable(false)
    .start()
}
