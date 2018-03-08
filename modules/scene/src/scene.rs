use std::any::TypeId;

use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::graphics::prelude::*;
use crayon::graphics::assets::prelude::*;
use crayon::resource::utils::prelude::*;
use crayon::utils::HashValue;

use components::prelude::*;
use assets::prelude::*;
use assets::material::MaterialParams;
use assets::pipeline::PipelineParams;
use graphics::renderer::Renderer;
use errors::*;

/// `Scene`s contain the environments of your game. Its relative easy to think of each
/// unique scene as a unique level. In each `Scene`, you place your envrionments,
/// obstacles, and decorations, essentially designing and building your game in pieces.
///
/// The `Scene` is arranged with simple tree hierarchy. A tree `Node` may have many children
/// but only a single parent, with the effect of a parent applied to all its child nodes.
/// A spatial `Transform` is associated with every tree node, the world transformation
/// could be calculated by concatenating such `Transform`s along the ancestors.
/// ```rust,ignore
/// let mut tree = scene.arena_mut::<Node>();
/// let mut transforms = scene.arena_mut::<Transform>();
/// Node::set_parent(&mut tree, child, parent)?;
/// Transform::set_world_position(&tree, &mut transforms, child, [1.0, 0.0, 0.0])?;
/// ```
///
/// And besides the spatial representation, `Element` is used to provide graphical data
/// that could be used to render on the screen. A `Element` could be one of `Camera`
/// `Lit` or `MeshRenderer`. Everytime you call the `Scene::render` with proper defined
/// scene, a list of drawcalls will be generated and submitted to `GraphicsSystem`.
///
/// ```rust,ignore
/// let _mesh_node = scene.build(MeshRenderer { ... });
/// let _lit_node = scene.build(Lit { ... });
/// let camera = Camera::perspective(math::Deg(60.0), 6.4 / 4.8, 0.1, 1000.0);
/// let camera_node = scene.build(camera);
/// self.scene.render(surface, camera_node)?;
/// ```
///
pub struct Scene {
    pub(crate) world: World,

    pub(crate) video: GraphicsSystemGuard,
    pub(crate) materials: Registery<MaterialParams>,
    pub(crate) pipelines: Registery<PipelineParams>,

    pub(crate) renderer: Renderer,
    pub(crate) fallback: Option<MaterialHandle>,
}

impl Scene {
    /// Creates a new `Scene`.
    pub fn new(ctx: &Context) -> Result<Self> {
        let video = GraphicsSystemGuard::new(ctx.shared::<GraphicsSystem>().clone());

        let mut world = World::new();
        world.register::<Node>();
        world.register::<Transform>();
        world.register::<Camera>();
        world.register::<Light>();
        world.register::<MeshRenderer>();

        let scene = Scene {
            world: world,
            video: video,

            pipelines: Registery::new(),
            materials: Registery::new(),
            fallback: None,

            renderer: Renderer::new(ctx)?,
        };

        Ok(scene)
    }

    /// Immutably borrows the arena of component. The borrow lasts until the returned
    /// `Fetch` exits scope. Multiple immutable borrows can be taken out at the same
    /// time.
    ///
    /// # Panics
    ///
    /// - Panics if user has not register the arena with type `T`.
    /// - Panics if the value is currently mutably borrowed.
    #[inline]
    pub fn arena<T>(&self) -> Fetch<T>
    where
        T: Component,
    {
        self.world.arena::<T>()
    }

    /// Mutably borrows the wrapped arena. The borrow lasts until the returned
    /// `FetchMut` exits scope. The value cannot be borrowed while this borrow
    /// is active.
    ///
    /// # Panics
    ///
    /// - Panics if user has not register the arena with type `T`.
    /// - Panics if the value is currently borrowed.
    #[inline]
    pub fn arena_mut<T>(&self) -> FetchMut<T>
    where
        T: Component,
    {
        self.world.arena_mut::<T>()
    }

    /// Build a new `Entity` in this scene.
    pub fn build(&mut self) -> EntityBuilder {
        self.world
            .build()
            .with_default::<Node>()
            .with_default::<Transform>()
    }

    pub fn create(&mut self) -> Entity {
        self.build().finish()
    }

    pub fn update<O, T>(&mut self, handle: Entity, value: O) -> Option<T>
    where
        O: Into<Option<T>>,
        T: Component,
    {
        let id = TypeId::of::<T>();
        assert!(id != TypeId::of::<Node>() && id != TypeId::of::<Transform>());

        if let Some(value) = value.into() {
            self.world.add(handle, value)
        } else {
            self.world.remove::<T>(handle)
        }
    }

    /// Deletes a node and its descendants from the `Scene`.
    pub fn delete(&mut self, handle: Entity) -> Result<()> {
        let descendants: Vec<_> = Node::descendants(&self.arena::<Node>(), handle).collect();
        for v in descendants {
            self.world.free(v);
        }

        Node::remove_from_parent(&mut self.arena_mut::<Node>(), handle)?;
        self.world.free(handle);

        Ok(())
    }

    /// Lookups pipeline object from location.
    pub fn lookup_pipeline(&self, location: Location) -> Option<PipelineHandle> {
        self.pipelines.lookup(location).map(|v| v.into())
    }

    /// Creates a new pipeline object that indicates the whole render pipeline of `Scene`.
    pub fn create_pipeline(&mut self, setup: PipelineSetup) -> Result<PipelineHandle> {
        if let Some(handle) = self.lookup_pipeline(setup.location()) {
            self.pipelines.inc_rc(handle);
            return Ok(handle.into());
        }

        let (location, setup, links) = setup.into();
        let params = setup.params.clone();
        let shader = self.video.create_shader(setup)?;

        Ok(self.pipelines
            .create(location, PipelineParams::new(shader, params, links))
            .into())
    }

    /// Deletes a pipelie object.
    pub fn delete_pipeline(&mut self, handle: PipelineHandle) {
        self.pipelines.dec_rc(handle);
    }

    /// Creates a new material instance from shader.
    pub fn create_material(&mut self, setup: MaterialSetup) -> Result<MaterialHandle> {
        if let Some(po) = self.pipelines.get(*setup.pipeline) {
            let location = Location::unique("");
            let material =
                MaterialParams::new(setup.pipeline, setup.variables, po.shader_params.clone());
            Ok(self.materials.create(location, material).into())
        } else {
            Err(Error::PipelineHandleInvalid(setup.pipeline))
        }
    }

    /// Updates the uniform variable of material.
    pub fn update_material<T1, T2>(&mut self, h: MaterialHandle, f: T1, v: T2) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        if let Some(m) = self.materials.get_mut(*h) {
            m.bind(f, v)?;
            Ok(())
        } else {
            Err(Error::MaterialHandleInvalid(h))
        }
    }

    /// Deletes the material instance from `Scene`. Any meshes that associated with a
    /// invalid/deleted material handle will be drawed with a fallback material marked
    /// with purple color.
    #[inline]
    pub fn delete_material(&mut self, handle: MaterialHandle) {
        self.materials.dec_rc(handle);
    }

    /// Advance to next frame.
    pub fn advance(&mut self, camera: Entity) -> Result<()> {
        self.renderer.advance(&self.world, camera)?;
        Ok(())
    }

    /// Draws the underlaying depth buffer of shadow mapping pass. This is used for
    /// debugging.
    pub fn draw_shadow<T>(&mut self, surface: T) -> Result<()>
    where
        T: Into<Option<SurfaceHandle>>,
    {
        self.renderer.draw_shadow(surface.into())
    }

    /// Renders objects into `Surface` from `Camera`.
    pub fn draw(&mut self, camera: Entity) -> Result<()> {
        if self.fallback.is_none() {
            let undefined = factory::pipeline::undefined(self)?;
            self.fallback = Some(self.create_material(MaterialSetup::new(undefined))?);
        }

        self.renderer.draw(self, camera)?;
        Ok(())
    }
}
