/*
    "Sunset" by @XorDev
    fragcoord.xyz
*/

struct Uniforms {
    size: vec2f,
    cursor: vec2f,
    time: f32,
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct Inputs {
    brightness: f32,
    color_speed: f32,
    wave_amp: f32,
    color_base: f32,
}
@group(1) @binding(0) var<uniform> inputs: Inputs;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4f {
    let x = f32(i32(idx) / 2) * 4.0 - 1.0;
    let y = f32(i32(idx) % 2) * 4.0 - 1.0;
    return vec4f(x, y, 0.0, 1.0);
}

const RGB: vec3f = vec3f(0.0, 1.0, 2.0);
const COLOR_WAVE: f32 = 14.0;
const COLOR_DOT: vec3f = vec3f(1.0, -1.0, 0.0);

const WAVE_STEPS: i32 = 8;
const WAVE_FREQ: f32 = 5.0;
const WAVE_EXP: f32 = 1.8;
const WAVE_VELOCITY: vec3f = vec3f(0.2);

const PASSTHROUGH: f32 = 0.2;
const SOFTNESS: f32 = 0.005;
const STEPS: i32 = 20;
const SKY: f32 = 10.0;
const FOV: f32 = 1.0;

@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let fragCoord = pos.xy;
    var z = 0.0;
    var d = 0.0;
    var s = 0.0;

    let dir = normalize(vec3f(
        2.0 * fragCoord - uniforms.size,
        -FOV * uniforms.size.y
    ));

    var col = vec3f(0.0);

    for (var i = 0; i < STEPS; i++) {
        var p = z * dir;

        var f = WAVE_FREQ;
        for (var j = 0; j < WAVE_STEPS; j++) {
            p += inputs.wave_amp * sin(p * f - WAVE_VELOCITY * uniforms.time).yzx / f;
            f *= WAVE_EXP;
        }

        s = 0.3 - abs(p.y);
        d = SOFTNESS + max(s, -s * PASSTHROUGH) / 4.0;
        z += d;

        let phase = COLOR_WAVE * s + dot(p, COLOR_DOT) + inputs.color_speed * uniforms.time;
        col += (cos(phase - RGB) + inputs.color_base) * exp(s * SKY) / d;
    }

    col *= SOFTNESS / f32(STEPS) * inputs.brightness;
    col = tanh(col * col);
    return vec4f(col, 1.0);
}
