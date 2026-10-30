// Brutus's Vengar view, the passes after a camera's picture is drawn (src/bin/play_vengar.rs):
//  - the world pass (FUN_000e66d0, vision mode 2): four taps of the picture zoomed toward the
//    screen's middle (vertex shader 0x6d: scales 1, 0.985, 0.97, 0.955 about c18.w, read as 0.5),
//    blended as pixel shader 0x45's first three stages do (by c0.a 0.5, then 0.66, then 0.75), made
//    grey by the dot with (0.9, 0.9, 0.5) (stage 3), the darks tinted (0, 0, 0.5) by
//    (1 - (2g)^2)^2 (stages 4-6 and the final combiner), and laid over the picture at the final
//    combiner's alpha 0.7 (SRCALPHA / INVSRCALPHA). All in the frame buffer's values (sRGB), as
//    the console's register combiners work; the game reads a half-size copy of the picture
//    (h_0796c3b0, 320 x 240), which each tap's 2 x 2 samples stand in for;
//  - the heat figures' halo (the camera that draws them): the figures' own colour spread round
//    their outline, in eight directions at three steps of 0.006 of the screen (the glare's 24
//    offset copies, FUN_000e47b0 with 0.006 at 0x40cfc8), added outside the figures; the weight is
//    fitted to the takes;
//  - the view's colour fade (FUN_000c6980: 0.6 s, level 1, colour ff 00 00): the picture times
//    the fade colour mixed toward white by the level;
//  - the OrgSen's view (vision mode 1, #21; FUN_000e6a70 in the world's place, FUN_000a45a0 for the
//    scan lines after the figures): the world pass maps the picture's luminance onto a thermal
//    palette, and the scan lines, on both cameras, multiply the picture by a cycle every four lines
//    of the game's 480. The materials aren't in the data (the vision object's, FUN_000a42d0, are
//    made of their classes' defaults): the palette and the lines are fitted to take10 of
//    todo/18-ammo-pickups (its measurements are at `thermal` and `scan_lines`).

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Pass {
    // on the world camera: 1 while the view is Vengar's, 2 while it's the OrgSen's
    world: f32,
    // the colour fade's level, 1 at a switch falling to 0
    fade: f32,
    // the halo's weight per tap on the heat camera (0: none)
    glow: f32,
    // the view's width over its height
    aspect: f32,
    // 1: the OrgSen's scan lines (both cameras)
    scan: f32,
    // the scan lines' cycle in the game's lines, its phase in cycles, and the lines in the view
    period: f32,
    phase: f32,
    rows: f32,
};

@group(0) @binding(0) var screen: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: Pass;

fn to_gamma(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((max(c, vec3<f32>(0.0)) + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

// The picture at `uv` (sRGB values).
fn at(uv: vec2<f32>) -> vec3<f32> {
    return to_gamma(textureSample(screen, screen_sampler, uv).rgb);
}

// One tap of the zoom: the picture `scale` of the way from the screen's middle to `uv`, as the
// half-size copy has it (a 2 x 2 average, half a game pixel either side; 640 x 480 game pixels).
fn tap(uv: vec2<f32>, scale: f32) -> vec3<f32> {
    let p = vec2<f32>(0.5) + (uv - vec2<f32>(0.5)) * scale;
    let d = vec2<f32>(0.5 / 640.0, 0.5 / 480.0);
    return 0.25 * (at(p + vec2<f32>(-d.x, -d.y)) + at(p + vec2<f32>(d.x, -d.y)) + at(p + vec2<f32>(-d.x, d.y)) + at(p + d));
}

// The OrgSen's thermal map (vision mode 1's world pass): the picture's luminance (the sRGB values
// the console works in) onto a palette of dark blue-purple rising to violet-magenta. The stops are
// the mean colour of take10's picture (sdm_e13, Tex's half, the OrgSen on, the six capture rows of
// a scan cycle averaged, which the cycle's mean 0.635 darkens: divided by it here) binned by the
// luminance of the same view a few seconds before, the camera still: (38, 22, 68) at black, blue
// 85 to 157 and red 44 to 96 through the greys, and the lava's bright orange (luminance over 0.7)
// saturating at (237, 64, 239). That binning wasn't a clean function (the vision lights change the
// lighting: the correlation 0.6), so the stops are the monotone envelope of it; the lights'
// strength is play_vengar.rs's VISION_LIGHT fit. The luminance is raised to THERMAL_GAMMA first:
// the demo's vision-lit terrain is darker than take10's (the blue's 10th / 50th / 90th percentile
// 46 / 53 / 92 against 43 / 72 / 104, the scan lines' cycle averaged), and 0.75 puts them at 54 /
// 71 / 106.
const THERMAL_GAMMA: f32 = 0.75;

fn thermal(l: f32) -> vec3<f32> {
    var stops = array<vec4<f32>, 9>(
        vec4<f32>(0.00, 38.0, 22.0, 68.0),
        vec4<f32>(0.08, 44.0, 22.0, 85.0),
        vec4<f32>(0.16, 58.0, 23.0, 118.0),
        vec4<f32>(0.26, 77.0, 23.0, 146.0),
        vec4<f32>(0.36, 96.0, 24.0, 160.0),
        vec4<f32>(0.50, 132.0, 35.0, 192.0),
        vec4<f32>(0.62, 167.0, 46.0, 217.0),
        vec4<f32>(0.72, 230.0, 63.0, 239.0),
        vec4<f32>(1.00, 239.0, 65.0, 239.0));
    let x = pow(clamp(l, 0.0, 1.0), THERMAL_GAMMA);
    for (var i = 0; i < 8; i++) {
        let a = stops[i];
        let b = stops[i + 1];
        if x <= b.x {
            return mix(a.yzw, b.yzw, (x - a.x) / (b.x - a.x)) / 255.0;
        }
    }
    return stops[8].yzw / 255.0;
}

// The scan lines' factor at `uv_y`: the picture times 0.635 + 0.365 sin(2 pi (y / period - phase)), y
// the line of the game's 480 (take10: a cycle of six capture rows, 0.27 darkest, 1.0 brightest).
fn scan_lines(uv_y: f32) -> f32 {
    let y = uv_y * settings.rows;
    return 0.635 + 0.365 * sin(6.2831853 * (y / settings.period - settings.phase));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let here = textureSample(screen, screen_sampler, in.uv);
    var c = to_gamma(here.rgb);
    if settings.world > 1.5 {
        c = thermal(dot(c, vec3<f32>(0.299, 0.587, 0.114)));
    } else if settings.world > 0.5 {
        let t0 = tap(in.uv, 1.0);
        let t1 = tap(in.uv, 0.985);
        let t2 = tap(in.uv, 0.97);
        let t3 = tap(in.uv, 0.955);
        // stages 0-2: lerps by c0.a 0.5, 0.66 and c0.rgb 0.75
        let b = 0.75 * (0.66 * (0.5 * t0 + 0.5 * t1) + 0.34 * t2) + 0.25 * t3;
        // stage 3: the grey; 4: twice it; 5: 1 - its square; 6: that squared (each clamped to
        // the combiners' -1..1)
        let g = clamp(dot(b, vec3<f32>(0.9, 0.9, 0.5)), -1.0, 1.0);
        let r1 = clamp(2.0 * g, -1.0, 1.0);
        let r0 = clamp(clamp(1.0 - r1 * r1, -1.0, 1.0) * clamp(1.0 - r1 * r1, -1.0, 1.0), 0.0, 1.0);
        // the final combiner (its inputs taken unsigned): r0 x the tinted grey + (1 - r0) x the grey
        let grey = max(g, 0.0);
        let out = r0 * vec3<f32>(0.0, 0.0, 0.5) * grey + (1.0 - r0) * vec3<f32>(grey);
        c = 0.7 * out + 0.3 * c;
    }
    if settings.glow > 0.0 {
        var sum = vec3<f32>(0.0);
        let step = vec2<f32>(0.006 * (4.0 / 3.0) / settings.aspect, 0.006);
        for (var k = 0; k < 8; k++) {
            let a = f32(k) * 0.7853982;
            let dir = vec2<f32>(sin(a), cos(a));
            for (var r = 1; r <= 3; r++) {
                sum += at(in.uv + dir * step * f32(r));
            }
        }
        c += (1.0 - here.a) * settings.glow * sum;
    }
    c *= mix(vec3<f32>(1.0), vec3<f32>(1.0, 0.0, 0.0), settings.fade);
    if settings.scan > 0.5 {
        c *= scan_lines(in.uv.y);
    }
    return vec4<f32>(to_linear(c), here.a);
}
