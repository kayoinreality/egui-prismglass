// Dual-Kawase 8-tap tent-filter upsample shader (linear Rgba16Float).

struct KawaseUniforms {
    half_pixel: vec2<f32>, // 0.5 / dst_size
    offset: f32,           // filter spread multiplier (typically 1.0..1.5)
    _pad: f32,
};

@group(0) @binding(0) var<uniform> params: KawaseUniforms;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var src_sampler: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    let x = f32((idx & 1u) << 2u) - 1.0;
    let y = 1.0 - f32((idx & 2u) << 1u);
    var out: VsOut;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let hp = params.half_pixel * params.offset;
    let uv = in.uv;

    var sum = textureSample(src_tex, src_sampler, uv + vec2<f32>(-hp.x * 2.0, 0.0));
    sum += textureSample(src_tex, src_sampler, uv + vec2<f32>(-hp.x, hp.y)) * 2.0;
    sum += textureSample(src_tex, src_sampler, uv + vec2<f32>(0.0, hp.y * 2.0));
    sum += textureSample(src_tex, src_sampler, uv + vec2<f32>(hp.x, hp.y)) * 2.0;
    sum += textureSample(src_tex, src_sampler, uv + vec2<f32>(hp.x * 2.0, 0.0));
    sum += textureSample(src_tex, src_sampler, uv + vec2<f32>(hp.x, -hp.y)) * 2.0;
    sum += textureSample(src_tex, src_sampler, uv + vec2<f32>(0.0, -hp.y * 2.0));
    sum += textureSample(src_tex, src_sampler, uv + vec2<f32>(-hp.x, -hp.y)) * 2.0;

    return sum / 12.0;
}
