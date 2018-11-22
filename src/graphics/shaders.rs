
pub const VERTEX_SHADER_SRC: &[u8] = b"
#version 400

uniform mat4 mv_matrix;
uniform mat4 proj_matrix;

in vec3 position;
in vec4 color;
in vec3 normal;

out VS_OUT {
    vec3 world_pos;
    vec4 color;
    vec3 world_normal;
} vs_out;

void main() {
    vs_out.world_pos = vec3(position);
    vs_out.color = color;
    vs_out.world_normal = normalize(normal);

    vec4 local_pos = mv_matrix * vec4(position, 1.0);
	gl_Position = proj_matrix * local_pos;
}
\0";

pub const FRAGMENT_SHADER_SRC: &[u8] = b"
#version 400

uniform mat4 mv_matrix;
uniform vec3 light_world_pos;

out vec4 frag_color;

in VS_OUT {
    vec3 world_pos;
    vec4 color;
    vec3 world_normal;
} fs_in;

void main() {
    // Phong shading
    vec3 color_in = vec3(fs_in.color);
    vec3 ambient_color = 0.3 * color_in;
    vec3 light_dir = normalize(light_world_pos - fs_in.world_pos);
    float diffuse_atten = max(dot(light_dir, fs_in.world_normal), 0);
    vec3 diffuse_color = diffuse_atten * color_in;
    
    // specular highlights
    vec3 eye_pos = vec3(-mv_matrix[3]);
    vec3 eye_dir = normalize(eye_pos - fs_in.world_pos);
    vec3 reflection_dir = reflect(-light_dir, fs_in.world_normal);
    float specular_atten = max(dot(eye_dir, reflection_dir), 0);
    vec3 highlight_color = vec3(1.0, 1.0, 1.0);
    vec3 specular_color =
        pow(specular_atten, 40) * highlight_color * color_in;

    vec3 result_color = ambient_color + diffuse_color + specular_color;

	frag_color = vec4(result_color, 1.0);
}
\0";

