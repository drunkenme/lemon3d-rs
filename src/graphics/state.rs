/// Specifies what kind of primitives to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

/// Specify whether front- or back-facing polygons can be culled.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum CullFace {
    Nothing,
    Front,
    Back,
}

/// Define front- and back-facing polygons.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum FrontFaceOrder {
    Clockwise,
    CounterClockwise,
}

/// A pixel-wise comparison function.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Comparison {
    Never,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
    Always,
}

/// Specifies how incoming RGBA values (source) and the RGBA in framebuffer (destination)
/// are combined.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Equation {
    /// Adds source and destination. Source and destination are multiplied
    /// by blending parameters before addition.
    Add,
    /// Subtracts destination from source. Source and destination are
    /// multiplied by blending parameters before subtraction.
    Subtract,
    /// Subtracts source from destination. Source and destination are
    /// multiplied by blending parameters before subtraction.
    ReverseSubtract,
}

/// Blend values.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum BlendValue {
    SourceColor,
    SourceAlpha,
    DestinationColor,
    DestinationAlpha,
}

/// Blend factors.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum BlendFactor {
    Zero,
    One,
    Value(BlendValue),
    OneMinusValue(BlendValue),
}

/// A struct that encapsulate all the necessary render states.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RenderState {
    pub cull_face: CullFace,
    pub front_face_order: FrontFaceOrder,
    pub depth_test: Comparison,
    pub depth_write: bool,
    pub depth_write_offset: Option<(f32, f32)>,
    pub color_blend: Option<(Equation, BlendFactor, BlendFactor)>,
    pub color_write: (bool, bool, bool, bool), // pub program: Handle,
}

impl Default for RenderState {
    fn default() -> Self {
        RenderState {
            cull_face: CullFace::Back,
            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: Comparison::Always, // no depth test,
            depth_write: false, // no depth write,
            depth_write_offset: None,
            color_blend: None,
            color_write: (false, false, false, false),
        }
    }
}

impl RenderState {
    #[inline]
    pub fn with_cull_face(&mut self, cull_face: CullFace) -> &mut Self {
        self.cull_face = cull_face;
        self
    }

    #[inline]
    pub fn with_front_face_order(&mut self, front_face_order: FrontFaceOrder) -> &mut Self {
        self.front_face_order = front_face_order;
        self
    }

    #[inline]
    pub fn with_depth_write(&mut self, write: bool) -> &mut Self {
        self.depth_write = write;
        self
    }

    #[inline]
    pub fn with_depth_test(&mut self, test: Comparison) -> &mut Self {
        self.depth_test = test;
        self
    }

    #[inline]
    pub fn with_color_blend(&mut self,
                            blend: Option<(Equation, BlendFactor, BlendFactor)>)
                            -> &mut Self {
        self.color_blend = blend;
        self
    }

    #[inline]
    pub fn with_color_write(&mut self,
                            red: bool,
                            green: bool,
                            blue: bool,
                            alpha: bool)
                            -> &mut Self {
        self.color_write = (red, green, blue, alpha);
        self
    }
}