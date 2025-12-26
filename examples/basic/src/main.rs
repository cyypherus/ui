use ui::*;

#[derive(Debug, Clone, Default)]
struct State {
    text_a: TextState,
    text_b: TextState,
    // toggle: ToggleState,
    // slider: SliderState,
    button: ButtonState,
    dropdown: DropdownState,
}

fn main() {
    App::builder(
        State {
            text_a: TextState {
                text: "Bio-luminescenct moss carpets power floating gardens while crystal-infused mycelium networks whisper data through the canopy above"
                    .to_string(),
                editing: false,
            },
            text_b: TextState {
                text: "With reverent whispers the fauna lift their gaze to the shafts of light piercing the deep green"
                    .to_string(),
                editing: false,
            },
            // toggle: ToggleState::default(),
            // slider: SliderState::default(),
            button: ButtonState::default(),
            dropdown: DropdownState::default(),
        },
        |state, app| {

                column_spaced(
                    20.,
                    vec![
                        // text(
                        //     id!(),
                        //     "Mycelial Networks Harmonize with Quantum-Grown Algae Towers",
                        // )
                        // .font_weight(FontWeight::BOLD)
                        // .font_size(30)
                        // .wrap()
                        // .finish(app),
                        // text_field(id!(), binding!(state, State, text_a)).wrap().finish(app),
                        text_field(id!(), binding!(state, State, text_b)).font_size(14).align(parley::Alignment::Left).wrap().finish(app),
                        // toggle(id!(), binding!(state, State, toggle)).finish(app).height(40.),
                        // slider(id!(), binding!(state, State, slider)).finish(app).height(40.),
                        // dropdown(id!(), binding!(state, State, dropdown), vec![
                        //     text(id!(), "Luminescent Moss"),
                        //     text(id!(), "Crystal Mycelium"),
                        //     text(id!(), "Quantum Algae"),
                        //     text(id!(), "Floating Gardens"),
                        //     text(id!(), "Cerebral Forests"),
                        //     text(id!(), "Glass Marrow"),
                        // ]).finish(app).height(20.),
                        // button(id!(), binding!(state, State, button)).text_label("Engage thrusters").finish(app).height(50.),
                    ],
                )
                .pad(20.)

        },
    )
    .inner_size(800, 600)
    // .resizable(false)
    .start()
}
