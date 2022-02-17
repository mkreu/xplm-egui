#version 140
uniform sampler2D u_sampler;

in vec4 v_rgba;
in vec2 v_tc;
out vec4 f_color;

void main() {
    // The texture sampler is sRGB aware, and OpenGL already expects linear rgba output
    // so no need for any sRGB conversions here:
    //gl_FragColor = v_rgba * texture2D(u_sampler, v_tc);
    //f_color = vec4(0.2f, 1.0f, 0.5f, 1.0f);
    f_color = v_rgba * texture2D(u_sampler, v_tc);
}
