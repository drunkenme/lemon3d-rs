use errors::*;
use math::prelude::{Aabb2, Vector2};
use utils::prelude::{DataBuffer, DataBufferPtr, HashValue};

use super::super::assets::prelude::*;
use super::Visitor;

type VarsPtr = DataBufferPtr<[(HashValue<str>, UniformVariable)]>;
type BytesPtr = DataBufferPtr<[u8]>;

#[derive(Debug, Clone)]
pub enum Command {
    Bind(SurfaceHandle),
    Draw(ShaderHandle, MeshHandle, MeshIndex, VarsPtr),
    UpdateScissor(SurfaceScissor),
    UpdateViewport(SurfaceViewport),

    CreateSurface(SurfaceHandle, SurfaceParams),
    DeleteSurface(SurfaceHandle),

    CreateShader(ShaderHandle, ShaderParams, String, String),
    DeleteShader(ShaderHandle),

    CreateTexture(TextureHandle, TextureParams, Option<TextureData>),
    UpdateTexture(TextureHandle, Aabb2<u32>, BytesPtr),
    DeleteTexture(TextureHandle),

    CreateRenderTexture(RenderTextureHandle, RenderTextureParams),
    DeleteRenderTexture(RenderTextureHandle),

    CreateMesh(MeshHandle, MeshParams, Option<MeshData>),
    UpdateVertexBuffer(MeshHandle, usize, BytesPtr),
    UpdateIndexBuffer(MeshHandle, usize, BytesPtr),
    DeleteMesh(MeshHandle),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Frame {
    pub cmds: Vec<Command>,
    pub bufs: DataBuffer,
}

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    /// Creates a new frame with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Frame {
            cmds: Vec::with_capacity(16),
            bufs: DataBuffer::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.cmds.clear();
        self.bufs.clear();
    }

    /// Dispatch frame tasks and draw calls to the backend context.
    pub fn dispatch(
        &mut self,
        visitor: &mut Visitor,
        dimensions: Vector2<u32>,
    ) -> Result<(u32, u32)> {
        unsafe {
            visitor.advance()?;

            let (mut dc, mut tris) = (0, 0);
            for v in self.cmds.drain(..) {
                match v {
                    Command::Bind(surface) => {
                        visitor.bind(surface, dimensions)?;
                    }

                    Command::Draw(shader, mesh, mesh_index, ptr) => {
                        let vars = self.bufs.as_slice(ptr);
                        dc += 1;
                        tris += visitor.draw(shader, mesh, mesh_index, vars)?;
                    }

                    Command::UpdateScissor(scissor) => {
                        visitor.update_surface_scissor(scissor)?;
                    }

                    Command::UpdateViewport(view) => {
                        visitor.update_surface_viewport(view)?;
                    }

                    Command::CreateSurface(handle, params) => {
                        visitor.create_surface(handle, params)?;
                    }

                    Command::DeleteSurface(handle) => {
                        visitor.delete_surface(handle)?;
                    }

                    Command::CreateShader(handle, params, vs, fs) => {
                        visitor.create_shader(handle, params, &vs, &fs)?;
                    }

                    Command::DeleteShader(handle) => {
                        visitor.delete_shader(handle)?;
                    }

                    Command::CreateTexture(handle, params, data) => {
                        visitor.create_texture(handle, params, data)?;
                    }

                    Command::UpdateTexture(handle, area, ptr) => {
                        let data = self.bufs.as_slice(ptr);
                        visitor.update_texture(handle, area, data)?;
                    }

                    Command::DeleteTexture(handle) => {
                        visitor.delete_texture(handle)?;
                    }

                    Command::CreateRenderTexture(handle, params) => {
                        visitor.create_render_texture(handle, params)?;
                    }

                    Command::DeleteRenderTexture(handle) => {
                        visitor.delete_render_texture(handle)?;
                    }

                    Command::CreateMesh(handle, params, data) => {
                        visitor.create_mesh(handle, params, data)?;
                    }

                    Command::UpdateVertexBuffer(handle, offset, ptr) => {
                        let data = self.bufs.as_slice(ptr);
                        visitor.update_vertex_buffer(handle, offset, data)?;
                    }

                    Command::UpdateIndexBuffer(handle, offset, ptr) => {
                        let data = self.bufs.as_slice(ptr);
                        visitor.update_index_buffer(handle, offset, data)?;
                    }

                    Command::DeleteMesh(handle) => {
                        visitor.delete_mesh(handle)?;
                    }
                }
            }

            visitor.flush()?;
            self.cmds.clear();
            Ok((dc, tris))
        }
    }
}
