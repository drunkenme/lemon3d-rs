//! Immutable or dynamic 2D texture.

use math;
use resource::utils::location::Location;
use video::errors::{Error, Result};

/// A texture is a container of one or more images. It can be the source of a texture
/// access from a Shader.
#[derive(Debug, Clone, Default)]
pub struct TextureSetup<'a> {
    pub location: Location<'a>,
    pub params: TextureParams,
    pub data: Option<&'a [u8]>,
}

impl<'a> TextureSetup<'a> {
    pub fn validate(&self) -> Result<()> {
        if self.location.is_shared() {
            if self.params.hint != TextureHint::Immutable {
                return Err(Error::CreateMutableSharedObject);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextureParams {
    pub format: TextureFormat,
    pub address: TextureAddress,
    pub filter: TextureFilter,
    pub hint: TextureHint,
    pub mipmap: bool,
    pub dimensions: math::Vector2<u32>,
}

impl_handle!(TextureHandle);

impl Default for TextureParams {
    fn default() -> Self {
        TextureParams {
            format: TextureFormat::U8U8U8U8,
            address: TextureAddress::Clamp,
            filter: TextureFilter::Linear,
            hint: TextureHint::Immutable,
            mipmap: false,
            dimensions: math::Vector2::new(0, 0),
        }
    }
}

/// A `RenderTexture` object is basicly texture object with special format. It can
/// be used as a render target. If the `sampler` field is true, it can also be ther
/// source of a texture access from a __shader__.
///
#[derive(Debug, Copy, Clone)]
pub struct RenderTextureSetup {
    pub format: RenderTextureFormat,
    pub address: TextureAddress,
    pub filter: TextureFilter,
    pub dimensions: math::Vector2<u32>,
    pub sampler: bool,
}

impl Default for RenderTextureSetup {
    fn default() -> Self {
        RenderTextureSetup {
            format: RenderTextureFormat::RGB8,
            address: TextureAddress::Clamp,
            filter: TextureFilter::Linear,
            dimensions: math::Vector2::new(0, 0),
            sampler: true,
        }
    }
}

pub type RenderTextureParams = RenderTextureSetup;
impl_handle!(RenderTextureHandle);

/// Hint abouts the intended update strategy of the data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureHint {
    /// The resource is initialized with data and cannot be changed later, this
    /// is the most common and most efficient usage. Optimal for render targets
    /// and resourced memory.
    Immutable,
    /// The resource is initialized without data, but will be be updated by the
    /// CPU in each frame.
    Stream,
    /// The resource is initialized without data and will be written by the CPU
    /// before use, updates will be infrequent.
    Dynamic,
}

/// Specify how the texture is used whenever the pixel being sampled.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFilter {
    /// Returns the value of the texture element that is nearest (in Manhattan distance)
    /// to the center of the pixel being textured.
    Nearest,
    /// Returns the weighted average of the four texture elements that are closest to the
    /// center of the pixel being textured.
    Linear,
}

/// Sets the wrap parameter for texture.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureAddress {
    /// Samples at coord x + 1 map to coord x.
    Repeat,
    /// Samples at coord x + 1 map to coord 1 - x.
    Mirror,
    /// Samples at coord x + 1 map to coord 1.
    Clamp,
    /// Same as Mirror, but only for one repetition.
    MirrorClamp,
}

/// List of all the possible formats of renderable texture which could be use as
/// attachment of framebuffer.
///
/// Each element of `Depth` is a single depth value. The `Graphics` converts it to
/// floating point, multiplies by the signed scale factor, adds the signed bias, and
/// clamps to the range [0,1].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RenderTextureFormat {
    RGB8,
    RGBA4,
    RGBA8,
    Depth16,
    Depth24,
    Depth32,
    Depth24Stencil8,
}

/// List of all the possible formats of input data when uploading to texture.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFormat {
    U8,
    U8U8,
    U8U8U8,
    U8U8U8U8,
    U5U6U5,
    U4U4U4U4,
    U5U5U5U1,
    U10U10U10U2,
    F16,
    F16F16,
    F16F16F16,
    F16F16F16F16,
    F32,
    F32F32,
    F32F32F32,
    F32F32F32F32,
}

impl TextureFormat {
    /// Returns the number of components of this client format.
    pub fn components(&self) -> u8 {
        match *self {
            TextureFormat::F32 | TextureFormat::F16 | TextureFormat::U8 => 1,
            TextureFormat::U8U8 | TextureFormat::F16F16 | TextureFormat::F32F32 => 2,
            TextureFormat::U5U6U5
            | TextureFormat::U8U8U8
            | TextureFormat::F16F16F16
            | TextureFormat::F32F32F32 => 3,
            TextureFormat::U8U8U8U8
            | TextureFormat::U4U4U4U4
            | TextureFormat::U5U5U5U1
            | TextureFormat::U10U10U10U2
            | TextureFormat::F16F16F16F16
            | TextureFormat::F32F32F32F32 => 4,
        }
    }

    /// Returns the size in bytes of a pixel of this type.
    pub fn size(&self) -> u8 {
        match *self {
            TextureFormat::U8 => 1,
            TextureFormat::U8U8
            | TextureFormat::U5U6U5
            | TextureFormat::U4U4U4U4
            | TextureFormat::U5U5U5U1
            | TextureFormat::F16 => 2,
            TextureFormat::U8U8U8 => 3,
            TextureFormat::U8U8U8U8
            | TextureFormat::U10U10U10U2
            | TextureFormat::F16F16
            | TextureFormat::F32 => 4,
            TextureFormat::F16F16F16 => 6,
            TextureFormat::F16F16F16F16 | TextureFormat::F32F32 => 8,
            TextureFormat::F32F32F32 => 12,
            TextureFormat::F32F32F32F32 => 16,
        }
    }
}