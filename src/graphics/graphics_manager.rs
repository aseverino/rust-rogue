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

use macroquad::material::{MaterialParams, load_material};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

fn setup_color_replacement_material() -> Result<Material, macroquad::Error> {
    let pipeline_params = PipelineParams {
        // this will do: out = src * src_alpha + dst * (1 - src_alpha)
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        // 2) disable depth writes so that transparent quads don't block later draws
        depth_write: false,
        ..Default::default()
    };

    let palette_material = load_material(
        ShaderSource::Glsl {
            vertex: include_str!("../../assets/shaders/default.vert"), //DEFAULT_VERTEX_SHADER,
            fragment: include_str!("../../assets/shaders/color_replace.frag"),
        },
        MaterialParams {
            pipeline_params,
            uniforms: vec![
                UniformDesc::new("SourceColor1", UniformType::Float4),
                UniformDesc::new("TargetColor1", UniformType::Float4),
                UniformDesc::new("SourceColor2", UniformType::Float4),
                UniformDesc::new("TargetColor2", UniformType::Float4),
            ],
            ..Default::default()
        },
    )?;

    Ok(palette_material)
}

pub fn set_color_replacement_uniforms(material: &mut Material) {
    material.set_uniform("SourceColor1", Vec4::new(1.0, 0.0, 0.0, 1.0)); // red
    material.set_uniform("TargetColor1", Vec4::new(0.0, 1.0, 1.0, 1.0)); // cyan
    material.set_uniform("SourceColor2", Vec4::new(0.0, 1.0, 0.0, 1.0)); // green
    material.set_uniform("TargetColor2", Vec4::new(1.0, 1.0, 0.0, 1.0)); // yellow
}

pub struct GraphicsManager {
    color_replace_material: Material,
}

impl GraphicsManager {
    pub fn new() -> Self {
        let color_replace_material =
            setup_color_replacement_material().expect("Failed to load color replacement material");

        Self {
            color_replace_material,
        }
    }

    pub fn get_color_replace_material(&self) -> &Material {
        &self.color_replace_material
    }
}
