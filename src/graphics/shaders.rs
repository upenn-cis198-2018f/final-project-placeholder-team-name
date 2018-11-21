
pub const VERTEX_SHADER_SRC: &[u8] = b"
#version 400

uniform mat4 mv_matrix;
uniform mat4 proj_matrix;

in vec4 position;
in vec4 color;
in vec3 normal;

out vec4 out_color;

out VS_OUT {
    vec4 color;
    vec3 normal;
} vs_out;

void main() {
	gl_Position = proj_matrix * mv_matrix * position;
    vec3 light_dir = vec3(1, 1, 1);
    float atten = 0.2 + clamp(dot(normal, light_dir), 0.0, 1.0);
    atten = min(atten, 1.0);
    vec4 out_color = atten * color;
    vs_out.color = out_color;
    vs_out.normal = normal;
}
\0";

pub const FRAGMENT_SHADER_SRC: &[u8] = b"
#version 400

out vec4 frag_color;

in VS_OUT {
    vec4 color;
    vec3 normal;
} fs_in;

void main() {
	frag_color = fs_in.color;
}
\0";

