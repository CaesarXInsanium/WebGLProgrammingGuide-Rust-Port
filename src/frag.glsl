#version 450 core

in vec3 ourColor;
in vec2 TexCoord;

layout (location = 0) out vec4 FragColor;


//layout (location = 0) uniform sampler2D ourTexture00;
// uniform sampler2D ourTexture01;

vec2 texture_coord_flip(vec2 coord){
  return vec2(coord.x, abs(1 - coord.y));
}

void main(){
  // flip the texture coordinates over the y axis
  vec2 new_tex = texture_coord_flip(TexCoord);

  // FragColor = mix(
  //   vec4(ourColor, 1.0),
  //   texture(ourTexture00, new_tex),
  //   0.5
  // );
 FragColor = vec4(ourColor, 1.0);
}
