use ui::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderInputs {
    brightness: f32,
    color_speed: f32,
    wave_amp: f32,
    color_base: f32,
}

#[derive(Debug, Clone, Default)]
struct State {
    text_a: TextState,
    text_b: TextState,
    toggle: ToggleState,
    button: ButtonState,
    dropdown: DropdownState,
}

const CAUSTICS_SHADER: &str = include_str!("../shaders/caustics.wgsl");

fn preset(state: &State) -> ShaderInputs {
    match state.dropdown.selected {
        1 => ShaderInputs {
            brightness: 0.8,
            color_speed: 0.1,
            wave_amp: 0.9,
            color_base: 2.0,
        },
        2 => ShaderInputs {
            brightness: 1.5,
            color_speed: 0.8,
            wave_amp: 0.3,
            color_base: 1.0,
        },
        3 => ShaderInputs {
            brightness: 1.0,
            color_speed: 0.5,
            wave_amp: 1.2,
            color_base: 1.2,
        },
        4 => ShaderInputs {
            brightness: 0.6,
            color_speed: 0.2,
            wave_amp: 0.4,
            color_base: 2.5,
        },
        5 => ShaderInputs {
            brightness: 2.0,
            color_speed: 1.0,
            wave_amp: 0.8,
            color_base: 0.8,
        },
        _ => ShaderInputs {
            brightness: 1.2,
            color_speed: 0.3,
            wave_amp: 0.6,
            color_base: 1.5,
        },
    }
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
            toggle: ToggleState::default(),
            // slider: SliderState::default(),
            button: ButtonState::default(),
            dropdown: DropdownState::default(),
        },
        |state, app| {

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
                        .finish(app.ctx()),
                        shader(id!(), CAUSTICS_SHADER)
                            .inputs(preset(state))
                            .corner_rounding(12.)
                            .finish(app.ctx())
                            .height(200.),
                        text_field(id!(), binding!(state, State, text_a)).wrap().finish(app.ctx()),
                        text_field(id!(), binding!(state, State, text_b)).font_size(14).align(parley::Alignment::Left).wrap().finish(app.ctx()),
                        toggle(id!(), binding!(state, State, toggle)).finish(app.ctx()).height(40.),
                        button(id!(), binding!(state, State, button)).text_label("Engage thrusters").finish(app.ctx()).height(50.),
                        // slider(id!(), binding!(state, State, slider)).finish(app.ctx()).height(40.),
                        dropdown(id!(), binding!(state, State, dropdown), vec![
                            text(id!(), "Luminescent Moss"),
                            text(id!(), "Crystal Mycelium"),
                            text(id!(), "Quantum Algae"),
                            text(id!(), "Floating Gardens"),
                            text(id!(), "Cerebral Forests"),
                            text(id!(), "Glass Marrow"),
                        ]).finish(app.ctx()).height(20.),
                    ],
                )
                .pad(20.)

        },
    )
    .inner_size(800, 600)
    // .resizable(false)
    .start()
}
