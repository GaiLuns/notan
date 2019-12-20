use nae_core::resources::{BaseTexture, Resource, ResourceConstructor, TextureFormat, TextureFilter};
use nae_core::BaseApp;
use crate::{TextureKey, GlContext};
use std::rc::Rc;
use std::cell::RefCell;
use crate::context::Context2d;
use glow::HasContext;

pub struct Texture {
    inner: Rc<RefCell<InnerTexture>>,
}

impl Texture {
    pub(crate) fn tex(&self) -> Option<TextureKey> {
        self.inner.borrow().tex
    }
}

impl BaseTexture for Texture {
    type Context2d = Context2d;

    fn width(&self) -> f32 {
        self.inner.borrow().width as _
    }

    fn height(&self) -> f32 {
        self.inner.borrow().height as _
    }

    fn from_size<T: BaseApp<Graphics=Self::Context2d>>(app: &mut T, width: i32, height: i32) -> Result<Self, String> {
        <Texture as BaseTexture>::from(app, width, height, TextureFormat::Rgba, TextureFormat::Rgba, TextureFilter::Nearest, TextureFilter::Nearest)
    }

    fn from<T: BaseApp<Graphics=Self::Context2d>>(app: &mut T, width: i32, height: i32, internal_format: TextureFormat, format: TextureFormat, min_filter: TextureFilter, mag_filter: TextureFilter) -> Result<Self, String> {
        let mut inner = InnerTexture::empty(width, height);
        let bpp = byte_per_pixel(internal_format, format);
        let tex = create_gl_tex_ext(
            &app.graphics().gl,
            width,
            height,
            &vec![0; (width * height) as usize * bpp],
            internal_format.into(),
            format.into(),
            min_filter.into(),
            mag_filter.into(),
            bpp,
        )?;
        inner.gl = Some(app.graphics().gl.clone());
        inner.tex = Some(tex);
        Ok(Self {
            inner: Rc::new(RefCell::new(inner)),
        })
    }

    fn format(&self) -> TextureFormat {
        self.inner.borrow().format
    }
}

impl Resource for Texture {
    fn parse<T: BaseApp>(&mut self, app: &mut T, data: Vec<u8>) -> Result<(), String> {
        unimplemented!()
    }

    fn is_loaded(&self) -> bool {
        unimplemented!()
    }
}

impl ResourceConstructor for Texture {
    fn new(file: &str) -> Self {
        unimplemented!()
    }
}

struct InnerTexture {
    width: i32,
    height: i32,
    raw: Vec<u8>,
    gl: Option<GlContext>,
    tex: Option<TextureKey>,
    format: TextureFormat,
}

impl InnerTexture {
    fn empty(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            raw: vec![],
            gl: None,
            tex: None,
            format: TextureFormat::Rgba,
        }
    }
}

impl Drop for InnerTexture {
    fn drop(&mut self) {
        if let (Some(gl), Some(tex)) = (self.gl.as_ref(), self.tex) {
            unsafe {
                gl.delete_texture(tex);
            }
        }
    }
}

//bytes_per_pixe table https://webgl2fundamentals.org/webgl/lessons/webgl-data-textures.html
fn byte_per_pixel(internal: TextureFormat, format: TextureFormat) -> usize {
    use TextureFormat::*;

    match (internal, format) {
        (R8, Red) => 1,
        _ => 4,
    }
}

fn create_gl_tex_ext(
    gl: &GlContext,
    width: i32,
    height: i32,
    data: &[u8],
    internal: i32,
    format: i32,
    min_filter: i32,
    mag_filter: i32,
    bytes_per_pixel: usize,
) -> Result<TextureKey, String> {
    unsafe {
        let tex = gl.create_texture()?;
        if bytes_per_pixel == 1 {
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
        }

        gl.bind_texture(glow::TEXTURE_2D, Some(tex));

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, mag_filter);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, min_filter);

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            internal,
            width,
            height,
            0,
            format as _,
            glow::UNSIGNED_BYTE,
            Some(data),
        );

        //TODO mipmaps? gl.generate_mipmap(glow::TEXTURE_2D);
        gl.bind_texture(glow::TEXTURE_2D, None);
        Ok(tex)
    }
}

impl From<TextureFilter> for u32 {
    fn from(f: TextureFilter) -> Self {
        use TextureFilter::*;
        match f {
            Linear => glow::LINEAR,
            Nearest => glow::NEAREST,
        }
    }
}

impl From<TextureFilter> for i32 {
    fn from(f: TextureFilter) -> Self {
        let f: u32 = f.into();
        f as _
    }
}

impl From<TextureFormat> for u32 {
    fn from(f: TextureFormat) -> Self {
        use TextureFormat::*;
        match f {
            Rgba => glow::RGBA,
            Red => glow::RED,
            R8 => glow::R8,
        }
    }
}

impl From<TextureFormat> for i32 {
    fn from(f: TextureFormat) -> Self {
        let f: u32 = f.into();
        f as _
    }
}