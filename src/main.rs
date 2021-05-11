use std::collections::HashMap;

// use anyhow::Result;

use crossfont::{
    BitmapBuffer, Error as RasterizerError, FontDesc, FontKey, GlyphKey, Rasterize, RasterizedGlyph, Rasterizer,
    Size as FontSize, Slant, Style, Weight,
};

use unicode_width::UnicodeWidthChar;

use glutin::{
    event::{Event as GlutinEvent, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window as GlutinWindow, WindowBuilder},
    ContextBuilder, ContextWrapper, NotCurrent, PossiblyCurrent,
};

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

const EIGHT: f32 = 8.0 / 255.0;

fn main() {
    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new().with_title("GL Font Things, Oof!");

    let windowed_context =
        make_current_context(ContextBuilder::new().build_windowed(window_builder, &event_loop).unwrap());

    gl::load_with(|c_ptr| windowed_context.get_proc_address(c_ptr) as *const _);

    event_loop.run(move |event, _window_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            GlutinEvent::LoopDestroyed => return,
            GlutinEvent::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                // WindowEvent::Focused(focused) => {}
                // WindowEvent::ReceivedCharacter(c) => {}
                WindowEvent::KeyboardInput { input, .. } if input.virtual_keycode.is_some() => {
                    match input.virtual_keycode.unwrap() {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
                _ => {}
            },
            GlutinEvent::RedrawRequested(_) => {
                clear_buffer();
                windowed_context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
}

fn clear_buffer() {
    unsafe {
        gl::ClearColor(EIGHT, EIGHT, EIGHT, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    };
}

fn make_current_context(
    windowed_context: ContextWrapper<NotCurrent, GlutinWindow>,
) -> ContextWrapper<PossiblyCurrent, GlutinWindow> {
    unsafe { windowed_context.make_current().unwrap() }
}

// Built using Alacritty as a reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Size(FontSize);

impl Default for Size {
    fn default() -> Self {
        Self(FontSize::new(11.))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Font {
    offset: Vector2<i8>,
    glyph_offset: Vector2<i8>,
    use_thin_strokes: bool,
    normal: String,
    pub size: Size,
}

impl Font {
    fn size(&self) -> FontSize {
        self.size.0
    }
}

#[derive(Debug, Clone, Copy)]
struct Glyph {
    text_id: gl::types::GLuint,
    multicolor: bool,
    top: i16,
    left: i16,
    width: i16,
    height: i16,
    uv_bot: f32,
    uv_left: f32,
    uv_width: f32,
    uv_height: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Vector2<T: Default> {
    x: T,
    y: T,
}

struct GlyphCache {
    cache: HashMap<GlyphKey, Glyph>,
    rasterizer: Rasterizer,
    font_key: FontKey,
    font_size: FontSize,
    glyph_offset: Vector2<i8>,
    metrics: crossfont::Metrics,
}

trait LoadGlyph {
    fn load_glyph(&mut self, rasterized: &RasterizedGlyph) -> Glyph;
    fn clear(&mut self);
}

#[allow(dead_code)]
impl GlyphCache {
    fn new<L: LoadGlyph>(
        mut rasterizer: Rasterizer,
        font: &Font,
        loader: &mut L,
    ) -> Result<GlyphCache, crossfont::Error> {
        let regular = Self::compute_font_keys(font, &mut rasterizer)?;

        rasterizer.get_glyph(GlyphKey { font_key: regular, character: 'm', size: font.size() })?;

        let metrics = rasterizer.metrics(regular, font.size())?;

        let mut cache = Self {
            cache: HashMap::default(),
            rasterizer,
            font_size: font.size(),
            font_key: regular,
            glyph_offset: font.glyph_offset,
            metrics,
        };

        cache.load_glyphs_for_font(regular, loader);

        Ok(cache)
    }

    fn load_glyphs_for_font<L: LoadGlyph>(&mut self, font: FontKey, loader: &mut L) {
        let size = self.font_size;

        for i in 32u8..=126u8 {
            self.get(GlyphKey { font_key: font, character: i as char, size }, loader, true);
        }
    }

    fn compute_font_keys(font: &Font, rasterizer: &mut Rasterizer) -> Result<FontKey, crossfont::Error> {
        let size = font.size();

        let regular_desc = Self::make_desc(&font.normal, Slant::Normal, Weight::Normal);

        let regular = Self::load_regular_font(rasterizer, &regular_desc, size)?;

        Ok(regular)
    }

    fn load_regular_font(
        rasterizer: &mut Rasterizer,
        description: &FontDesc,
        size: FontSize,
    ) -> Result<FontKey, crossfont::Error> {
        match rasterizer.load_font(description, size) {
            Ok(font) => Ok(font),
            Err(err) => {
                eprintln!("{}", err);

                let fallback_desc = Self::make_desc(&Font::default().normal, Slant::Normal, Weight::Normal);
                rasterizer.load_font(&fallback_desc, size)
            }
        }
    }

    fn make_desc(desc: &String, slant: Slant, weight: Weight) -> FontDesc {
        FontDesc::new(desc, Style::Description { slant, weight })
    }

    fn get<L: LoadGlyph>(&mut self, glyph_key: GlyphKey, loader: &mut L, show_missing: bool) -> Glyph {
        // try and load from cache
        if let Some(glyph) = self.cache.get(&glyph_key) {
            return *glyph;
        };

        // rasterize
        let glyph = match self.rasterizer.get_glyph(glyph_key) {
            Ok(rasterized) => self.load_glyph(loader, rasterized),
            Err(RasterizerError::MissingGlyph(rasterized)) if show_missing => {
                // use '\0' as "missing" to cache only once
                let missing_key = GlyphKey { character: '\0', ..glyph_key };
                if let Some(glyph) = self.cache.get(&missing_key) {
                    *glyph
                } else {
                    // if no missing glyph, insert as '\0'
                    let glyph = self.load_glyph(loader, rasterized);
                    self.cache.insert(missing_key, glyph);

                    glyph
                }
            }
            Err(_) => self.load_glyph(loader, Default::default()),
        };

        *self.cache.entry(glyph_key).or_insert(glyph)
    }

    fn load_glyph<L: LoadGlyph>(&self, loader: &mut L, mut glyph: RasterizedGlyph) -> Glyph {
        glyph.left += i32::from(self.glyph_offset.x);
        glyph.top += i32::from(self.glyph_offset.y);
        glyph.top -= self.metrics.descent as i32;

        if glyph.character.width() == Some(0) {
            glyph.left += self.metrics.average_advance as i32;
        }

        loader.load_glyph(&glyph)
    }
}
