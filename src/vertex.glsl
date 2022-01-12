#version 330 

attribute vec2 aPos;
attribute float aPointSize;

void main(){
	gl_PointSize = aPointSize;
	gl_Position = vec4(aPos, 0,1);
	//gl_Position = aPos;
}
