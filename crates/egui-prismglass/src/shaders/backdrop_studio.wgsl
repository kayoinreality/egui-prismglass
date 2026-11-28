// Vibrant 8K Silk & Architectural 3D Studio Backdrop shader for `egui-prismglass`.
//
// Inspired by Apple Liquid Glass (iOS 26 / macOS Tahoe): sweeping razor-sharp diagonal
// silk folds with luminous crisp crest edges and deep, vibrant studio color gradients.
// Because the diagonal fold edge is razor-sharp outside cards, Snell lens refraction
// inside every Liquid Glass card & pill bends the fold with unmistakable physical realism.

struct BackdropUniforms {
    canvas_top: vec4<f32>,       // linear RGB
    canvas_bottom: vec4<f32>,    // linear RGB
    orb_primary: vec4<f32>,      // linear RGB
    orb_secondary: vec4<f32>,    // linear RGB
    orb_accent: vec4<f32>,       // linear RGB
    ext_rect_uv: vec4<f32>,      // [u_min, v_min, u_max, v_max] for optional sub-rect external texture
    screen_size: vec2<f32>,      // physical pixels
    time_secs: f32,
    intensity: f32,
    scene_id: u32,               // 0 = Studio Silk, 1 = Liquid Prism, 2 = Architectural Grid, 3 = Studio Neutral, 4 = None, 5 = Aurora
    is_dark: u32,
    target_is_srgb: u32,
    _pad: u32,
};

@group(0) @binding(0) var<uniform> u: BackdropUniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) idx: u32) -> VsOut {
    let x = f32((idx & 1u) << 2u) - 1.0;
    let y = 1.0 - f32((idx & 2u) << 1u);
    var out: VsOut;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn ign(p: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(0.06711056 * p.x + 0.00583715 * p.y));
}

fn gamma_from_linear_rgb(rgb: vec3<f32>) -> vec3<f32> {
    let clamped = max(rgb, vec3<f32>(0.0));
    let cutoff = clamped < vec3<f32>(0.0031308);
    let lower = clamped * vec3<f32>(12.92);
    let higher = vec3<f32>(1.055) * pow(clamped, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(higher, lower, cutoff);
}

// Smooth silk fold surface shading (3D undulating velvet/satin wave with specular ridge).
fn silk_fold_shade(d: f32, width: f32) -> vec2<f32> {
    // Returns (soft_highlight, deep_trough)
    let highlight = exp(-pow(d / width, 2.0) * 2.2);
    let trough = exp(-pow((d - width * 0.55) / (width * 0.65), 2.0) * 1.9);
    return vec2<f32>(highlight, trough);
}

// Scene 1: Liquid Prism — Apple iOS 26 / macOS Tahoe Vibrant Indigo-Cobalt Silk Fold!
// Rich electric indigo-violet upper-left field sliced by a razor-sharp diagonal illuminated
// silk crest descending into deep velvety sapphire & cobalt silk waves.
fn render_liquid_prism(uv: vec2<f32>) -> vec3<f32> {
    let px = 1.0 / max(u.screen_size.y, 1.0);
    let t = u.time_secs * 0.14;
    let k = u.intensity;

    // Sweeping diagonal primary silk ridge running from bottom-left (0.14, 1.0) to top-right (0.94, 0.0)
    // with a graceful organic S-curve so glass cards crossing it showcase dramatic Snell refraction!
    let diag_line = (1.0 - uv.y) - (uv.x * 1.18 - 0.14 + 0.035 * sin(uv.x * 3.4 + t));
    // Positive = upper-left violet field; negative = lower-right deep cobalt silk waves
    let aa = px * 1.6;
    let lower_mask = 1.0 - smoothstep(-aa, aa, diag_line);

    if u.is_dark == 1u {
        // --- Upper-left field: vibrant velvety electric indigo / royal violet ---
        let ul_grad = clamp(uv.x * 0.35 + uv.y * 0.65, 0.0, 1.0);
        let indigo_deep = vec3<f32>(0.036, 0.016, 0.44);   // rich #3421B0-like linear
        let indigo_bright = vec3<f32>(0.11, 0.068, 0.62);  // vibrant #5C46D6-like linear
        let lavender_glow = vec3<f32>(0.24, 0.21, 0.72);
        var upper = mix(indigo_deep, indigo_bright, smoothstep(0.0, 0.85, ul_grad));

        // Soft glowing silk bloom just above the diagonal ridge
        let upper_bloom = exp(-max(diag_line, 0.0) * 4.2);
        upper = mix(upper, lavender_glow, upper_bloom * 0.42 * k);

        // Soft royal-blue sweep in far bottom-left corner (matches reference image composition)
        let bl_glow = exp(-dot(uv - vec2<f32>(0.05, 0.92), uv - vec2<f32>(0.05, 0.92)) * 5.2);
        upper = mix(upper, vec3<f32>(0.012, 0.095, 0.52), bl_glow * 0.75 * k);

        // --- Lower-right field: sculpted velvety deep sapphire & electric cobalt silk folds ---
        let dist_below = max(-diag_line, 0.0);

        // Secondary sweeping 3D silk folds curving across the lower-right
        let fold1_d = dist_below - (0.20 + 0.045 * sin(uv.x * 4.2 - 0.5 + t));
        let fold2_d = dist_below - (0.50 + 0.055 * cos(uv.x * 3.2 + 0.8 - t * 0.7));
        let f1 = silk_fold_shade(fold1_d, 0.14);
        let f2 = silk_fold_shade(fold2_d, 0.19);

        let midnight_navy = vec3<f32>(0.002, 0.010, 0.075);
        let royal_cobalt = vec3<f32>(0.009, 0.068, 0.44);
        let electric_blue = vec3<f32>(0.032, 0.165, 0.68);
        let sky_crest = vec3<f32>(0.36, 0.56, 0.98);

        var lower = mix(midnight_navy, royal_cobalt, clamp(uv.x * 0.55 + uv.y * 0.45, 0.0, 1.0));
        // Deep velvet shadow right under the primary ridge lip for dramatic 3D overhang contrast
        let under_lip_shadow = exp(-pow((dist_below - 0.12) / 0.11, 2.0));
        lower *= 1.0 - under_lip_shadow * 0.72 * k;

        // Illuminated glossy silk crest right along the upper edge of the lower fold
        let ridge_sheen = exp(-dist_below * 8.5);
        lower = mix(lower, electric_blue, clamp(ridge_sheen * 0.78 + f1.x * 0.55 + f2.x * 0.65, 0.0, 0.92) * k);
        lower *= 1.0 - (f1.y * 0.62 + f2.y * 0.35) * k;

        // Luminous ice-blue sheen catching the upper bevel of the main fold
        let crest_band = exp(-dist_below * 28.0);
        lower += sky_crest * crest_band * 0.52 * k;

        var col = mix(upper, lower, lower_mask);

        // Razor-sharp 2px luminous ice-white highlight along the exact diagonal silk crease!
        // This crisp high-contrast crease is what bends dramatically inside Liquid Glass bezels!
        let crease_core = exp(-pow(diag_line / (px * 3.2), 2.0));
        let crease_halo = exp(-pow((diag_line + 0.008) / 0.018, 2.0)) * lower_mask;
        col += vec3<f32>(0.55, 0.72, 1.0) * (crease_core * 0.78 + crease_halo * 0.25) * k;

        return col;
    } else {
        // Light mode Liquid Prism: crisp alabaster-lilac & sky-cobalt silk fold
        let upper_base = mix(vec3<f32>(0.90, 0.92, 0.99), vec3<f32>(0.80, 0.85, 0.98), clamp(uv.y * 0.8 + uv.x * 0.2, 0.0, 1.0));
        let dist_below = max(-diag_line, 0.0);
        let fold1_d = dist_below - (0.24 + 0.04 * sin(uv.x * 4.0 + t));
        let f1 = silk_fold_shade(fold1_d, 0.16);

        var lower = mix(vec3<f32>(0.56, 0.72, 0.98), vec3<f32>(0.76, 0.86, 0.99), clamp(uv.y * 0.7 + uv.x * 0.3, 0.0, 1.0));
        lower = mix(lower, vec3<f32>(0.36, 0.56, 0.96), f1.y * 0.45 * k);
        lower = mix(lower, vec3<f32>(0.96, 0.98, 1.0), clamp(exp(-dist_below * 16.0) * 0.75 + f1.x * 0.35, 0.0, 0.85) * k);

        var col = mix(upper_base, lower, lower_mask);
        let crease_core = exp(-pow(diag_line / (px * 3.0), 2.0));
        col = mix(col, vec3<f32>(1.0, 1.0, 1.0), crease_core * 0.85 * k);
        return col;
    }
}

// Scene 0: Studio Silk (SculptedWaves) — Sleek 8K diagonal obsidian/sapphire or pearl-sky silk.
// Calibrated for Onyx Dark (deep luxury carbon-sapphire) and Frost Light (crisp airy pearl-blue).
fn render_studio_silk(uv: vec2<f32>) -> vec3<f32> {
    let px = 1.0 / max(u.screen_size.y, 1.0);
    let t = u.time_secs * 0.14;
    let k = u.intensity;

    let diag_line = (1.0 - uv.y) - (uv.x * 1.12 - 0.10 + 0.04 * sin(uv.x * 3.1 + t));
    let aa = px * 1.6;
    let lower_mask = 1.0 - smoothstep(-aa, aa, diag_line);
    let dist_below = max(-diag_line, 0.0);

    if u.is_dark == 1u {
        // Onyx Dark: deep smoked carbon-indigo upper field + razor-sharp titanium-cobalt silk fold
        let upper_base = mix(vec3<f32>(0.010, 0.013, 0.028), vec3<f32>(0.022, 0.032, 0.072), clamp(uv.x * 0.4 + uv.y * 0.6, 0.0, 1.0));
        let upper_bloom = exp(-max(diag_line, 0.0) * 5.0);
        let upper = mix(upper_base, vec3<f32>(0.048, 0.072, 0.16), upper_bloom * 0.45 * k);

        let fold1_d = dist_below - (0.22 + 0.04 * sin(uv.x * 3.8 + t));
        let f1 = silk_fold_shade(fold1_d, 0.16);

        var lower = mix(vec3<f32>(0.004, 0.006, 0.014), vec3<f32>(0.016, 0.028, 0.068), clamp(uv.x * 0.6 + uv.y * 0.4, 0.0, 1.0));
        let under_lip = exp(-pow((dist_below - 0.11) / 0.10, 2.0));
        lower *= 1.0 - under_lip * 0.72 * k;
        lower += vec3<f32>(0.032, 0.062, 0.155) * (exp(-dist_below * 8.0) * 0.75 + f1.x * 0.55) * k;
        lower *= 1.0 - f1.y * 0.50 * k;

        var col = mix(upper, lower, lower_mask);
        let crease = exp(-pow(diag_line / (px * 3.0), 2.0));
        let rim_tint = mix(vec3<f32>(0.42, 0.56, 0.85), u.orb_accent.rgb * 0.75, 0.35);
        col += rim_tint * crease * 0.72 * k;
        return col;
    } else {
        // Frost Light: luminous pearl-white studio silk with crisp glacier-blue diagonal crest
        let upper = mix(vec3<f32>(0.93, 0.945, 0.985), vec3<f32>(0.85, 0.89, 0.97), clamp(uv.y * 0.75 + uv.x * 0.25, 0.0, 1.0));
        let fold1_d = dist_below - (0.24 + 0.04 * sin(uv.x * 3.8 + t));
        let f1 = silk_fold_shade(fold1_d, 0.17);

        var lower = mix(vec3<f32>(0.72, 0.81, 0.96), vec3<f32>(0.86, 0.91, 0.985), clamp(uv.x * 0.55 + uv.y * 0.45, 0.0, 1.0));
        lower = mix(lower, vec3<f32>(0.54, 0.69, 0.95), f1.y * 0.42 * k);
        lower = mix(lower, vec3<f32>(0.98, 0.99, 1.0), exp(-dist_below * 14.0) * 0.65 * k);

        var col = mix(upper, lower, lower_mask);
        let crease = exp(-pow(diag_line / (px * 2.8), 2.0));
        col = mix(col, vec3<f32>(1.0, 1.0, 1.0), crease * 0.82 * k);
        return col;
    }
}

// Scene 2: Architectural Grid — Crisp Linear/Figma precision canvas + vibrant diagonal highlight.
fn render_architectural_grid(uv: vec2<f32>) -> vec3<f32> {
    let px = 1.0 / max(u.screen_size.y, 1.0);
    let aspect = u.screen_size.x / max(u.screen_size.y, 1.0);
    let k = u.intensity;

    var col = render_liquid_prism(uv) * select(0.92, 0.55, u.is_dark == 1u);
    if u.is_dark == 0u {
        col = mix(vec3<f32>(0.93, 0.95, 0.985), col, 0.55);
    }

    let cell_px = 48.0;
    let pos_px = uv * u.screen_size;
    let g = abs(fract(pos_px / cell_px - 0.5) - 0.5) * cell_px;
    let line_minor = 1.0 - smoothstep(0.45, 1.25, min(g.x, g.y));

    let major_px = cell_px * 4.0;
    let gm = abs(fract(pos_px / major_px - 0.5) - 0.5) * major_px;
    let line_major = 1.0 - smoothstep(0.55, 1.35, min(gm.x, gm.y));

    let center = vec2<f32>(0.62, 0.48);
    let d_ring = length((uv - center) * vec2<f32>(aspect, 1.0));
    let ring_mid = 1.0 - smoothstep(px * 1.4, px * 2.8, abs(d_ring - 0.26));

    if u.is_dark == 1u {
        col += vec3<f32>(0.12, 0.16, 0.28) * (line_minor * 0.35 + line_major * 0.85) * k;
        col += vec3<f32>(0.34, 0.58, 1.0) * (ring_mid * 0.72) * k;
    } else {
        col = mix(col, vec3<f32>(0.42, 0.52, 0.72), clamp((line_minor * 0.16 + line_major * 0.36) * k, 0.0, 0.55));
        col = mix(col, vec3<f32>(0.18, 0.42, 0.96), clamp(ring_mid * 0.55 * k, 0.0, 0.70));
    }

    return col;
}

// Scene 3: Studio Neutral — Distraction-free architectural studio canvas (no diagonal bars or waves).
fn render_studio_neutral(uv: vec2<f32>) -> vec3<f32> {
    var col = mix(u.canvas_top.rgb, u.canvas_bottom.rgb, clamp(uv.y, 0.0, 1.0));
    let overhead = exp(-length((uv - vec2<f32>(0.5, 0.0)) * vec2<f32>(1.15, 1.9)) * 2.2);
    let wash = select(vec3<f32>(1.0, 1.0, 1.0), u.orb_primary.rgb, u.is_dark == 1u);
    let wash_strength = select(0.04, 0.045, u.is_dark == 1u) * u.intensity;
    col = mix(col, wash, clamp(overhead * wash_strength, 0.0, 0.12));
    return col;
}

// Scene 5: Aurora — soft multi-color light pools on a deep (or airy) base. The default wallpaper
// of the Prism presets. Pools are blended in gamma space so the result matches CSS radial
// gradients: violet (orb_primary), blue (orb_secondary), teal (orb_accent), pink (canvas_bottom).
fn aurora_weight(uv: vec2<f32>, center: vec2<f32>, radius: vec2<f32>, edge: f32) -> f32 {
    let d = length((uv - center) / radius);
    let w = clamp(1.0 - d / edge, 0.0, 1.0);
    return w * w * (3.0 - 2.0 * w);
}

fn to_gamma(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}

fn render_aurora(uv: vec2<f32>) -> vec3<f32> {
    let t = u.time_secs * 0.06;
    let k = clamp(u.intensity, 0.0, 1.5);
    let dark = u.is_dark == 1u;
    let a_violet = select(0.55, 0.60, dark) * k;
    let a_blue = select(0.55, 0.60, dark) * k;
    let a_teal = select(0.40, 0.50, dark) * k;
    let a_pink = select(0.40, 0.45, dark) * k;

    var col = to_gamma(u.canvas_top.rgb);
    let w_pink = aurora_weight(uv, vec2<f32>(0.0 + 0.02 * sin(t * 1.3), 1.0), vec2<f32>(0.45, 0.55), 0.60);
    col = mix(col, to_gamma(u.canvas_bottom.rgb), clamp(w_pink * a_pink, 0.0, 1.0));
    let w_teal = aurora_weight(uv, vec2<f32>(0.80 + 0.03 * cos(t), 1.0), vec2<f32>(0.55, 0.60), 0.62);
    col = mix(col, to_gamma(u.orb_accent.rgb), clamp(w_teal * a_teal, 0.0, 1.0));
    let w_blue = aurora_weight(uv, vec2<f32>(0.90 - 0.02 * sin(t), 0.05 + 0.02 * cos(t * 0.8)), vec2<f32>(0.50, 0.60), 0.60);
    col = mix(col, to_gamma(u.orb_secondary.rgb), clamp(w_blue * a_blue, 0.0, 1.0));
    let w_violet = aurora_weight(uv, vec2<f32>(0.10 + 0.03 * sin(t * 0.9), 0.15), vec2<f32>(0.60, 0.70), 0.62);
    col = mix(col, to_gamma(u.orb_primary.rgb), clamp(w_violet * a_violet, 0.0, 1.0));

    return pow(max(col, vec3<f32>(0.0)), vec3<f32>(2.2));
}

@fragment
fn fs_generate(in: VsOut) -> @location(0) vec4<f32> {
    var rgb: vec3<f32>;
    if u.scene_id == 1u {
        rgb = render_liquid_prism(in.uv);
    } else if u.scene_id == 2u {
        rgb = render_architectural_grid(in.uv);
    } else if u.scene_id == 5u {
        rgb = render_aurora(in.uv);
    } else if u.scene_id == 3u || u.scene_id == 4u {
        rgb = render_studio_neutral(in.uv);
    } else {
        rgb = render_studio_silk(in.uv);
    }
    return vec4<f32>(max(rgb, vec3<f32>(0.0)), 1.0);
}

// --- Screen Blit & External Sub-Rect Projection Entry Point ---
@group(1) @binding(0) var src_tex: texture_2d<f32>;
@group(1) @binding(1) var src_sampler: sampler;

@fragment
fn fs_blit(in: VsOut) -> @location(0) vec4<f32> {
    let rect_min = u.ext_rect_uv.xy;
    let rect_max = u.ext_rect_uv.zw;
    let inside = all(in.uv >= rect_min) && all(in.uv <= rect_max);
    let local_uv = clamp(
        (in.uv - rect_min) / max(rect_max - rect_min, vec2<f32>(1e-4)),
        vec2<f32>(0.0),
        vec2<f32>(1.0),
    );
    let sampled = textureSample(src_tex, src_sampler, local_uv).rgb;
    let neutral_bg = mix(u.canvas_top.rgb, u.canvas_bottom.rgb, clamp(in.uv.y, 0.0, 1.0));
    let linear_rgb = select(neutral_bg, sampled, inside);
    if u.target_is_srgb == 1u {
        return vec4<f32>(max(linear_rgb, vec3<f32>(0.0)), 1.0);
    }
    var gamma_rgb = gamma_from_linear_rgb(linear_rgb);
    let dither = (ign(in.pos.xy) - 0.5) / 255.0;
    gamma_rgb = clamp(gamma_rgb + vec3<f32>(dither), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(gamma_rgb, 1.0);
}
