#version 330 core
out vec4 FragColor;
  
in vec4 Color;
in vec2 TexCoords;

uniform sampler2D screenTexture;

void main()
{ 
    float k = texture(screenTexture, TexCoords).r;
    FragColor = vec4(Color.r, Color.g, Color.b, Color.a * k);
}