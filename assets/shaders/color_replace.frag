// SPDX-License-Identifier: MIT
//
// Copyright (c) 2025 Alexandre Severino
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D Texture;
uniform vec4 SourceColor1;
uniform vec4 TargetColor1;
uniform vec4 SourceColor2;
uniform vec4 TargetColor2;
uniform vec4 SourceColor3;
uniform vec4 TargetColor3;
uniform vec4 SourceColor4;
uniform vec4 TargetColor4;

void main() {
    vec4 tex_color = texture2D(Texture, uv);

    if (distance(tex_color.rgb, SourceColor1.rgb) < 0.10) {
        gl_FragColor = vec4(TargetColor1.rgb, tex_color.a);
    } else if (distance(tex_color.rgb, SourceColor2.rgb) < 0.01) {
        gl_FragColor = vec4(TargetColor2.rgb, tex_color.a);
    } else if (distance(tex_color.rgb, SourceColor3.rgb) < 0.01) {
        gl_FragColor = vec4(TargetColor3.rgb, tex_color.a);
    } else if (distance(tex_color.rgb, SourceColor4.rgb) < 0.01) {
        gl_FragColor = vec4(TargetColor4.rgb, tex_color.a);
    } else {
        gl_FragColor = tex_color;
    }
}