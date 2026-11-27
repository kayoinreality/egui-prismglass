// Apple Liquid Glass (iOS 26 / macOS Tahoe) Optical Composite Shader for `egui-prismglass`.
//
// Optical Architecture:
// 1. Analytic rounded-rect SDF with crisp sub-pixel AA (`fwidth`)
// 2. Physical convex-bezel Snell lens refraction: bends backdrop details inward around
//    rounded borders with sharp optical clarity along the glass lip (the signature Liquid Glass hook!)
// 3. Subtle RGB chromatic dispersion along curved bevels
// 4. Clean interior transmission with Beer-Lambert theme-tinted glass filtering & vibrancy
//    (zero milky glare over center text or icons!)
// 5. Dual opposing Fresnel rim speculars (crisp top-left key light + opposing bottom-right bounce)
//    strictly confined to the machined outer rim so typography and icons stay razor-sharp.

struct GlassCardUniforms {
    // Card rect in physical pixels: (min_x, min_y, width, height)
    rect_px: vec4<f32>,
    // (screen_width_px, screen_height_px, corner_radius_px, pixels_per_point)
    screen_and_geom: vec4<f32>,
    // (blur_radius_px, vibrancy, refraction_strength, chromatic_dispersion)
    optics: vec4<f32>,
    // (grain_intensity, border_width_px, prism_intensity, presence)
    surface_fx: vec4<f32>,
    // Linear RGB + straight alpha
    gradient_top: vec4<f32>,
    gradient_bottom: vec4<f32>,
    border: vec4<f32>,
    specular_top: vec4<f32>,
    specular_accent: vec4<f32>,
    inner_glow: vec4<f32>,
    // flags: (is_dark, target_is_srgb, pad, pad)
    flags: vec4<u32>,
};

@group(0) @binding(0) var<uniform> card: GlassCardUniforms;

@group(1) @binding(0) var tex_sharp: texture_2d<f32>;
@group(1) @binding(1) var tex_blur_med: texture_2d<f32>;
@group(1) @binding(2) var tex_blur_deep: texture_2d<f32>;
@group(1) @binding(3) var blur_sampler: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local_px: vec2<f32>,
    @location(1) card_uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var corner = vec2<f32>(0.0, 0.0);
    switch idx {
        case 0u: { corner = vec2<f32>(0.0, 0.0); }
        case 1u: { corner = vec2<f32>(1.0, 0.0); }
        case 2u: { corner = vec2<f32>(0.0, 1.0); }
        case 3u: { corner = vec2<f32>(1.0, 0.0); }
        case 4u: { corner = vec2<f32>(1.0, 1.0); }
        default: { corner = vec2<f32>(0.0, 1.0); }
    }

    let pad = 2.0;
    let min_px = card.rect_px.xy - vec2<f32>(pad);
    let size_px = card.rect_px.zw + vec2<f32>(pad * 2.0);
    let px = min_px + corner * size_px;

    let screen = max(card.screen_and_geom.xy, vec2<f32>(1.0));
    let ndc = vec2<f32>(
        (px.x / screen.x) * 2.0 - 1.0,
        1.0 - (px.y / screen.y) * 2.0,
    );

    let center = card.rect_px.xy + card.rect_px.zw * 0.5;
    let card_uv = (px - card.rect_px.xy) / max(card.rect_px.zw, vec2<f32>(1.0));

    var out: VsOut;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.local_px = px - center;
    out.card_uv = card_uv;
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

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

fn sd_rounded_box_normal(p: vec2<f32>, b: vec2<f32>, r: f32) -> vec2<f32> {
    let eps = 0.75;
    let dx = sd_rounded_box(p + vec2<f32>(eps, 0.0), b, r) - sd_rounded_box(p - vec2<f32>(eps, 0.0), b, r);
    let dy = sd_rounded_box(p + vec2<f32>(0.0, eps), b, r) - sd_rounded_box(p - vec2<f32>(0.0, eps), b, r);
    let g = vec2<f32>(dx, dy);
    let len = length(g);
    if len > 1e-5 {
        return g / len;
    }
    return vec2<f32>(0.0, -1.0);
}

// Continuous Dual-Kawase blur pyramid sample at `uv` for a physical blur radius.
fn sample_pyramid(uv: vec2<f32>, blur_px: f32) -> vec3<f32> {
    let clamped_uv = clamp(uv, vec2<f32>(0.001), vec2<f32>(0.999));
    let sharp = textureSample(tex_sharp, blur_sampler, clamped_uv).rgb;
    let med = textureSample(tex_blur_med, blur_sampler, clamped_uv).rgb;
    let deep = textureSample(tex_blur_deep, blur_sampler, clamped_uv).rgb;

    let t_med = clamp(blur_px / 14.0, 0.0, 1.0);
    let t_deep = clamp((blur_px - 12.0) / 30.0, 0.0, 1.0);
    let s_med = t_med * t_med * (3.0 - 2.0 * t_med);
    let s_deep = t_deep * t_deep * (3.0 - 2.0 * t_deep);

    let first = mix(sharp, med, s_med);
    return mix(first, deep, s_deep);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let half_size = card.rect_px.zw * 0.5;
    let min_half = min(half_size.x, half_size.y);
    let radius = clamp(card.screen_and_geom.z, 0.0, min_half);
    let ppp = max(card.screen_and_geom.w, 1.0);

    let d = sd_rounded_box(in.local_px, half_size, radius);
    let aa_width = max(fwidth(d), 1.0);
    let coverage = clamp(0.5 - d / aa_width, 0.0, 1.0);
    if coverage <= 0.001 {
        discard;
    }

    let screen = max(card.screen_and_geom.xy, vec2<f32>(1.0));
    let base_uv = in.pos.xy / screen;

    let normal = sd_rounded_box_normal(in.local_px, half_size, radius);
    let edge_dist = max(-d, 0.0);

    let blur_px = card.optics.x;
    let vibrancy = card.optics.y;
    let refract_strength = card.optics.z;
    let dispersion = card.optics.w;

    // --- 1. Physical Convex-Bezel Snell Lens Refraction ---
    // Bevel thickness scales with corner curvature (wider dome on pills, refined bezel on large cards).
    let bevel_width = clamp(
        min(min_half * 0.68, max(radius * 0.85, 18.0 * ppp)),
        14.0 * ppp,
        38.0 * ppp,
    );
    // x = 0 in flat interior, x = 1 at the outer glass rim.
    let x = clamp(1.0 - edge_dist / bevel_width, 0.0, 1.0);
    let x2 = x * x;

    // Convex squircle lens surface slope: gentle start inside, steep optical bend near the rim!
    // Creates the iconic Liquid Glass hook where backdrop lines curl smoothly into the glass edge.
    let lens_slope = clamp((x * x2) / sqrt(max(1.0 - x2 * 0.90, 0.10)) * 0.62, 0.0, 1.75);

    let max_refract_px = 42.0 * ppp * refract_strength;
    let uv_refract = -normal * (lens_slope * max_refract_px) / screen;

    // Liquid Glass Optical Clarity: crisp refractive edge along the curved bezel, soft creamy blur inside!
    let edge_clarity = clamp(pow(x, 1.35) * 0.88 * refract_strength, 0.0, 0.92);
    let local_blur_px = blur_px * (1.0 - edge_clarity);

    // Subtle prismatic RGB chromatic dispersion concentrated along the steep outer bevel curve.
    let disp_px = 5.5 * ppp * dispersion * lens_slope;
    let uv_disp = normal * (disp_px / screen);

    var bg: vec3<f32>;
    if dispersion > 0.005 && x > 0.02 {
        let uv_r = base_uv + uv_refract + uv_disp;
        let uv_g = base_uv + uv_refract;
        let uv_b = base_uv + uv_refract - uv_disp;
        bg = vec3<f32>(
            sample_pyramid(uv_r, local_blur_px).r,
            sample_pyramid(uv_g, local_blur_px).g,
            sample_pyramid(uv_b, local_blur_px).b,
        );
    } else {
        bg = sample_pyramid(base_uv + uv_refract, local_blur_px);
    }

    // --- 2. Vibrancy & Theme-Tinted Glass Transmission (Beer-Lambert) ---
    let luma = dot(bg, vec3<f32>(0.2126, 0.7152, 0.0722));
    var vibrant = max(mix(vec3<f32>(luma), bg, vibrancy), vec3<f32>(0.0));

    let v_grad = clamp(in.card_uv.y, 0.0, 1.0);
    let body_tint = mix(card.gradient_top, card.gradient_bottom, v_grad);

    // Measure tint chromaticity: neutral smoked glass vs vibrantly colored tinted glass (like ruby/cobalt/amber pills!).
    let tint_max = max(max(body_tint.r, body_tint.g), max(body_tint.b, 0.0001));
    let tint_min = min(min(body_tint.r, body_tint.g), body_tint.b);
    let tint_chroma = clamp((tint_max - tint_min) / tint_max, 0.0, 1.0);
    let norm_hue = clamp(body_tint.rgb / tint_max, vec3<f32>(0.06), vec3<f32>(1.0));

    // Optical Beer-Lambert filter: colored glass selectively transmits its own hue while letting
    // refracted background highlights create glowing internal caustics (exactly like colored Apple Liquid Glass pills!).
    let filter_strength = mix(0.22, 0.85, tint_chroma);
    let hue_filter = mix(vec3<f32>(1.0), norm_hue, filter_strength);

    var color: vec3<f32>;
    var surface_alpha: f32 = 1.0;

    if card.flags.z == 1u {
        // Translucent Overlay Lens mode: composite directly over the live `egui` framebuffer
        // via hardware ALPHA_BLENDING so underlying video stages/widgets shine through cleanly.
        color = body_tint.rgb;
        surface_alpha = clamp(body_tint.a * 1.35, 0.20, 0.56);
    } else if card.flags.x == 1u {
        // Dark mode glass: preserve vibrant backdrop light while keeping interior contrast crisp for white text.
        let transmitted = vibrant * hue_filter;
        // Boost caustic refracted light slightly along the curved bevel lip (looks like glowing liquid crystal!)
        let caustic_boost = 1.0 + lens_slope * (0.28 + 0.55 * tint_chroma) * refract_strength;
        color = mix(transmitted * caustic_boost, body_tint.rgb, clamp(body_tint.a, 0.0, 0.90));
        // Add luminous refracted caustic back into strongly colored tinted glass pills
        if tint_chroma > 0.25 {
            let refracted_luma = dot(bg, vec3<f32>(0.2126, 0.7152, 0.0722));
            let caustic_glow = norm_hue * pow(clamp(refracted_luma * 1.6, 0.0, 1.2), 1.35) * (0.25 + 0.65 * lens_slope);
            color += caustic_glow * tint_chroma * refract_strength;
        }
    } else {
        // Light mode glass: airy translucent crystal milk-glass that preserves clean dark text contrast.
        let transmitted = mix(vibrant * hue_filter, body_tint.rgb, 0.18);
        color = mix(transmitted, body_tint.rgb, clamp(body_tint.a, 0.0, 0.90));
    }

    // Subtle interior dark-edge depth right inside the outer lip (gives 3D physical thickness without clouding center!)
    let inner_depth = exp(-pow((edge_dist - 4.5 * ppp) / (4.0 * ppp), 2.0)) * refract_strength;
    if card.flags.x == 1u && card.flags.z == 0u {
        color *= 1.0 - inner_depth * 0.14;
    }

    // Tactile frosted micro-grain (subtle enough to stay crystal-clean and never muddy).
    let grain_amp = card.surface_fx.x;
    if grain_amp > 0.0005 {
        let noise = ign(in.pos.xy) - 0.5;
        let grain_scale = select(grain_amp * 0.42, grain_amp * 0.28, card.flags.x == 1u);
        color = max(color + vec3<f32>(noise * grain_scale), vec3<f32>(0.0));
    }

    // --- 3. Dual Opposing Liquid Glass Rim Speculars (ZERO Glare Over Text/Icons!) ---
    let border_w = max(card.surface_fx.y, 1.0 * ppp);
    let prism_k = card.surface_fx.z;
    let light_dir = normalize(vec2<f32>(-0.62, -0.78)); // crisp top-left studio light
    let key_dot = max(dot(normal, light_dir), 0.0);
    let opp_dot = max(dot(normal, -light_dir), 0.0);     // opposing bottom-right internal Fresnel catch!

    // Perimeter hairline glass edge (keeps silhouette crisp against any backdrop).
    let border_mask = 1.0 - smoothstep(border_w * 0.35, border_w * 1.25, edge_dist);
    let border_a = clamp(border_mask * card.border.a, 0.0, 0.95);
    if card.flags.z == 1u {
        let out_a = border_a + surface_alpha * (1.0 - border_a);
        color = (card.border.rgb * border_a + color * surface_alpha * (1.0 - border_a)) / max(out_a, 1e-4);
        surface_alpha = out_a;
    } else {
        color = mix(color, card.border.rgb, border_a);
    }

    // Signature Apple Liquid Glass dual-arc specular rim strictly confined to the outer 2.2px edge!
    let spec_rim = 1.0 - smoothstep(0.0, border_w * 2.1, edge_dist);
    let top_left_arc = pow(key_dot, 2.2);
    let bottom_right_arc = pow(opp_dot, 2.6) * 0.72;
    let combined_arc = clamp(top_left_arc + bottom_right_arc, 0.0, 1.0);
    let spec_alpha = clamp(spec_rim * combined_arc * card.specular_top.a * 1.35, 0.0, 0.96);
    if card.flags.z == 1u {
        let out_a = spec_alpha + surface_alpha * (1.0 - spec_alpha);
        color = (card.specular_top.rgb * spec_alpha + color * surface_alpha * (1.0 - spec_alpha)) / max(out_a, 1e-4);
        surface_alpha = out_a;
    } else {
        color = mix(color, card.specular_top.rgb, spec_alpha);
    }

    // Subtle soft Fresnel glow hugging ONLY the outer 5px curved bevel (never touches center text!).
    let bevel_halo = exp(-pow((edge_dist - border_w * 1.6) / (border_w * 2.8), 2.0));
    let halo_strength = (top_left_arc * 0.85 + bottom_right_arc * 0.65) * card.inner_glow.a;
    let halo_a = clamp(bevel_halo * halo_strength * 0.9, 0.0, 0.55);
    if card.flags.z == 1u {
        let out_a = halo_a + surface_alpha * (1.0 - halo_a);
        color = (card.inner_glow.rgb * halo_a + color * surface_alpha * (1.0 - halo_a)) / max(out_a, 1e-4);
        surface_alpha = out_a;
    } else {
        color = mix(color, card.inner_glow.rgb, halo_a);
    }

    // Prismatic edge accent sheen grazing the corner curvature.
    if prism_k > 0.01 {
        let prism_arc = pow(max(dot(normal, normalize(vec2<f32>(-0.74, -0.67))), 0.0), 3.2)
            + pow(max(dot(normal, normalize(vec2<f32>(0.74, 0.67))), 0.0), 3.6) * 0.55;
        let prism_alpha = clamp(spec_rim * prism_arc * card.specular_accent.a * prism_k, 0.0, 0.72);
        if card.flags.z == 1u {
            let out_a = prism_alpha + surface_alpha * (1.0 - prism_alpha);
            color = (card.specular_accent.rgb * prism_alpha + color * surface_alpha * (1.0 - prism_alpha)) / max(out_a, 1e-4);
            surface_alpha = out_a;
        } else {
            color = mix(color, card.specular_accent.rgb, prism_alpha);
        }
    }

    let final_alpha = clamp(surface_alpha * coverage * card.surface_fx.w, 0.0, 1.0);

    if card.flags.y == 1u {
        return vec4<f32>(max(color, vec3<f32>(0.0)), final_alpha);
    }
    var gamma_rgb = gamma_from_linear_rgb(color);
    let dither = (ign(in.pos.xy) - 0.5) / 255.0;
    gamma_rgb = clamp(gamma_rgb + vec3<f32>(dither), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(gamma_rgb, final_alpha);
}
