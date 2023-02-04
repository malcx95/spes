#version 450

#define CHUNK 80
#define LAYERS 6
#define PIXELATE 1

layout (location = 0) out vec4 color;

uniform vec2 window_dimensions;
uniform vec2 player;
uniform float global_scale;

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p+45.32);
    return fract(p.x*p.y);
}

// Draw a star at uv == (0, 0)
vec3 star(vec2 uv) {
    float dist = length(uv);
    float glow = 0.005 / dist;

    // float ray = max(0.0, 1.0 - abs(uv.x * uv.y * 500.0)) * smoothstep(0.4, 0.1, dist);

    // vec3 color = vec3(glow + ray);
    vec3 color = vec3(glow, glow, glow);

    return color;
}

vec3 star_layer(vec2 uv) {
    vec3 col = vec3(0);
    return col;
}

vec3 star_layer(vec2 gr, vec2 id) {
    vec3 color = vec3(0.0);

    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            vec2 offset = vec2(x, y);

            float h1 = hash(id + offset);
            float h2 = hash(id + offset + 1);

            float size = fract(h1 * 3123.43);
            vec3 star_color = star(gr - offset - vec2(h1 - 0.5, h2 - 0.5));
            star_color *= smoothstep(0.95, 1.0, size);
            color += star_color;
        }
    }

    return color;
}

void main() {
    vec2 uv = gl_FragCoord.xy;

    vec3 col = vec3(0);

    for (float i = (1.0 / LAYERS); i < 1; i += 1.0 / (LAYERS - 1)) {
        vec2 uvp = uv + (player * (100 * i)) + (vec2(77, 31) * (3 + i * 70));

        uvp = PIXELATE * floor(uvp / PIXELATE);

        vec2 id = floor(uvp / CHUNK);
        vec2 gr = fract(uvp / CHUNK) - 0.5;

        float scale = mix(0.5, 1, i);
        col += star_layer(gr, id) * scale * global_scale;
    }

    color = vec4(col, 1.0);
}
