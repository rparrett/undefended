#import bevy_sprite::mesh2d_view_bindings

struct StarfieldMaterial {
    pos: vec2<f32>,
    _wasm_padding: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> material: StarfieldMaterial;


fn hash22(p: vec2<f32>) -> vec2<f32> {
	var p3: vec3<f32> = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.103, 0.0973));
	p3 = p3 + (dot(p3, p3.yzx + 19.19));
	return fract((p3.xx + p3.yz) * p3.zy);
}

fn noise(p: vec2<f32>) -> f32 {
    var n: vec2<f32> = floor(p);
    var f: vec2<f32> = fract(p);

    var mg: vec2<f32>;
    var mr: vec2<f32>;

    var md: f32 = 8.0;
    for(var j: i32 = -1; j <= 1; j += 1) {
        for(var i: i32 = -1; i <= 1; i += 1) {
            var g: vec2<f32> = vec2(f32(i), f32(j));
            var o: vec2<f32> = hash22(n + g);

            var r: vec2<f32> = g + o - f;
            var d: f32 = dot(r, r);

            if(d < md) {
                md = d;
                mr = r;
                mg = g;
            }
        }
    }
    return md;
}

fn starfield(samplePosition: vec2<f32>, threshold: f32) -> vec3<f32> {
	let starValue: f32 = noise(samplePosition);
	var power: f32 = max(1. - starValue / threshold, 0.);
	power = power * power * power;

	return vec3<f32>(power);
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
	var finalColor: vec3<f32>;

    let pos = material.pos / vec2<f32>(-1000., 1000.);
    let threshold = 0.0003;

	for (var i: i32 = 1; i <= 7; i = i + 1) {
		let layer: f32 = f32(i);
		let inv: f32 = sqrt(1. / layer);

        let layer_offset = vec2<f32>(layer * 100., -layer * 50.);
        let layer_zoom = (1. + layer * 0.6) / 500.;
        let layer_speed = inv;
        let layer_brightness = inv * 0.4;

        let starfield_coords = (position.xy + layer_offset) * layer_zoom - pos * layer_speed;

		finalColor = finalColor + (starfield(starfield_coords, threshold) * layer_brightness);
	}

	let fragColor = vec4<f32>(finalColor, 1.);

    return fragColor;
}