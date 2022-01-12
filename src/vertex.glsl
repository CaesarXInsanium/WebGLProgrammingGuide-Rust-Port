#version 130

in vec2 aPos;

uniform mat4 uModelMatrix;


void main(){
	gl_Position = uModelMatrix * vec4(aPos, 0,1);
}
