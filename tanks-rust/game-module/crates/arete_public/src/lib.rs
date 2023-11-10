//! READ ONLY.
//!
//! This crate provides all the public types and traits of Arete.
//! It is a Rust wrapper on top of Arete's C API.

use std::{
    ffi::{c_char, c_int, c_void, CStr},
    marker::PhantomData,
    mem::{size_of, transmute_copy},
    panic::catch_unwind,
};

use game_module_macro::{Component, Resource};
use nalgebra_glm::Vec2;

pub use linalg::*;

mod linalg;

/// The version of Arete which this module is designed to support.
pub const ENGINE_VERSION: u32 = make_api_version(0, 1, 0);

pub const fn make_api_version(major: u32, minor: u32, patch: u32) -> u32 {
    ((major) << 25) | ((minor) << 15) | (patch)
}

pub const fn api_version_major(version: u32) -> u32 {
    version >> 25
}

pub const fn api_version_minor(version: u32) -> u32 {
    (version >> 15) & !(!0 << 10)
}

pub const fn api_version_patch(version: u32) -> u32 {
    version & !(!0 << 15)
}

pub const fn api_version_compatible(version: u32) -> bool {
    api_version_major(ENGINE_VERSION) == api_version_major(version)
        && api_version_minor(ENGINE_VERSION) == api_version_minor(version)
}

/// A handle identifying a component or resource type.
pub type ComponentId = u16;

/// A handle identifying a loaded asset.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssetId(pub u32);

/// A trait representing an ECS Component. All structs which are to be used as
/// a Component must `#[derive(Component)]`.
pub trait Component: Copy + Send + Sync + Sized {
    fn id() -> ComponentId;

    fn set_id(id: ComponentId);

    fn string_id() -> &'static CStr;
}

/// A trait representing an ECS Resource. All structs which are to be used as
/// a Resource must `#[derive(Resource)]`.
pub trait Resource: Send + Sync + Sized {
    fn new() -> Self;

    fn id() -> ComponentId;

    fn set_id(id: ComponentId);

    fn string_id() -> &'static CStr;
}

/// A handle representing an entity.
#[repr(transparent)]
#[derive(Component, PartialEq, Eq)]
pub struct EntityId(pub u64);

/// A resource repesenting the current input state.
#[repr(C)]
#[derive(Resource, Copy, Clone, Debug, Default)]
pub struct InputState {
    pub key_a: ButtonState,
    pub key_b: ButtonState,
    pub key_c: ButtonState,
    pub key_d: ButtonState,
    pub key_e: ButtonState,
    pub key_f: ButtonState,
    pub key_g: ButtonState,
    pub key_h: ButtonState,
    pub key_i: ButtonState,
    pub key_j: ButtonState,
    pub key_k: ButtonState,
    pub key_l: ButtonState,
    pub key_m: ButtonState,
    pub key_n: ButtonState,
    pub key_o: ButtonState,
    pub key_p: ButtonState,
    pub key_q: ButtonState,
    pub key_r: ButtonState,
    pub key_s: ButtonState,
    pub key_t: ButtonState,
    pub key_u: ButtonState,
    pub key_v: ButtonState,
    pub key_w: ButtonState,
    pub key_x: ButtonState,
    pub key_y: ButtonState,
    pub key_z: ButtonState,

    pub key_space: ButtonState,

    pub key_0: ButtonState,
    pub key_1: ButtonState,
    pub key_2: ButtonState,
    pub key_3: ButtonState,
    pub key_4: ButtonState,
    pub key_5: ButtonState,
    pub key_6: ButtonState,
    pub key_7: ButtonState,
    pub key_8: ButtonState,
    pub key_9: ButtonState,

    pub mouse: Mouse,

    pub touches: [TouchInput; MAX_TOUCHES],
    pub touches_len: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Mouse {
    pub cursor: Cursor,
    pub scroll_delta: Vec2,
    pub button_left: ButtonState,
    pub button_right: ButtonState,
    pub button_middle: ButtonState,
    pub is_present: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Cursor {
    pub position: ScreenPosition,
    /// Change in position from last frame (applies even when the cursor is out of frame).
    pub delta_position: ScreenPosition,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ButtonState {
    pub pressed: bool,
    pub pressed_this_frame: bool,
    pub released_this_frame: bool,
}

/// A unique touch identifier.
pub type TouchId = usize;

/// This is *twice* the number of physical touches, to account for the case
/// when five touches ended and five touches began in the same frame.
pub const MAX_TOUCHES: usize = 10;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TouchInput {
    pub touch_id: TouchId,
    pub phase: TouchPhase,
    pub position: ScreenPosition,
    /// Change in position from last frame.
    pub delta_position: ScreenPosition,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub enum TouchPhase {
    #[default]
    Began,
    Moved,
    Stationary,
    Ended,
}

/// Screen position in range `[0, 1]`, where top-left is `(0, 0)`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ScreenPosition {
    pub x: f32,
    pub y: f32,
}

impl InputState {
    /// A helper function returning all active touches.
    pub fn touches(&self) -> impl Iterator<Item = &TouchInput> {
        self.touches[..self.touches_len].iter()
    }

    /// A helper function returning only new touches.
    pub fn touches_began(&self) -> impl Iterator<Item = &TouchInput> {
        self.touches[..self.touches_len]
            .iter()
            .filter(|touch| matches!(touch.phase, TouchPhase::Began))
    }

    /// A helper function returning only moved touches.
    pub fn touches_moved(&self) -> impl Iterator<Item = &TouchInput> {
        self.touches[..self.touches_len]
            .iter()
            .filter(|touch| matches!(touch.phase, TouchPhase::Moved))
    }

    /// A helper function returning only ended touches.
    pub fn touches_ended(&self) -> impl Iterator<Item = &TouchInput> {
        self.touches[..self.touches_len]
            .iter()
            .filter(|touch| matches!(touch.phase, TouchPhase::Ended))
    }
}

/// A resource containing values which are constant for the whole frame.
#[repr(C)]
#[derive(Resource, Default, Clone, Copy)]
pub struct FrameConstants {
    pub delta_time: f32,
}

/// A component representing a 3D transform.
#[repr(C)]
#[derive(Component, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

/// A component representing a 3D camera.
#[repr(C)]
#[derive(Component, Debug)]
pub struct Camera {
    /// Vertical field-of-view, in radians.
    pub fov: f32,
    /// Near clip plane. A larger value results in less z-fighting at larger
    /// distances, but cannot render objects closer than the near plane.
    pub near_plane: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 1.0,
            near_plane: 0.1,
        }
    }
}

/// A component representing a normalized RGB color.
/// Values are in the range [0, 1], but values may exceed the upper bound.
#[repr(C)]
#[derive(Component, Debug)]
pub struct Color {
    pub val: Vec3,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            val: Vec3::new(1.0, 0.0, 1.0),
        }
    }
}

/// A component representing a directional (sun) light.
/// The first directional light spawned into the world will cast shadows.
#[repr(C, align(16))]
#[derive(Component, Debug, Default)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub intensity: Vec3,
}

/// A component representing a point light.
#[repr(C, align(16))]
#[derive(Component, Debug, Default)]
pub struct PointLight {
    pub position: Vec3,
    pub intensity: Vec3,
}

/// A resource representing the intensity of ambient light.
#[repr(C, align(16))]
#[derive(Resource, Debug)]
pub struct GlobalLighting {
    pub ambient_intensity: Vec3,
}

impl Default for GlobalLighting {
    fn default() -> Self {
        Self {
            ambient_intensity: Vec3::new(0.1, 0.1, 0.1),
        }
    }
}

/// A resource representing the game window size (in pixels).
#[repr(C)]
#[derive(Resource, Debug, Default)]
pub struct Aspect {
    pub x: f32,
    pub y: f32,
}

/// A resource with settings for the frame rate limit.
#[repr(C)]
#[derive(Resource, Debug)]
pub struct FrameRateSettings {
    pub frame_rate_limit: f64,
}

impl Default for FrameRateSettings {
    fn default() -> Self {
        Self {
            frame_rate_limit: f64::INFINITY,
        }
    }
}

/// A component representing a moveable static mesh.
#[repr(C)]
#[derive(Component, Debug)]
pub struct DynamicStaticMesh {
    pub asset_id: AssetId,
}

/// A generic reference to a component. This type is necessary to pass components to `Engine::spawn()`.
///
/// It is NOT recommended to use this struct manually -- use the `bundle!()` macro to automatically convert components.
#[repr(C)]
pub struct ComponentRef<'a> {
    pub component_id: ComponentId,
    pub component_size: usize,
    pub component_val: *const c_void,
    marker: PhantomData<&'a u8>,
}

impl<'a, C> From<&'a C> for ComponentRef<'a>
where
    C: Component,
{
    fn from(value: &'a C) -> Self {
        Self {
            component_id: C::id(),
            component_size: size_of::<C>(),
            component_val: value as *const C as *const c_void,
            marker: PhantomData,
        }
    }
}

impl<'a, C> From<&'a mut C> for ComponentRef<'a>
where
    C: Component,
{
    fn from(value: &'a mut C) -> Self {
        (&*value).into()
    }
}

/// Converts a list of component references into the type which `Engine::spawn()` accepts.
#[macro_export]
macro_rules! bundle {
    ($($c:expr),* $(,)?) => (
        &[$(::arete_public::ComponentRef::from($c)),*]
    )
}

/// A resource which exposes engine functionality, like spawning and despawning.
#[repr(C)]
#[derive(Resource)]
pub struct Engine {
    engine_handle: *const c_void,
    spawn: unsafe extern "C" fn(*const c_void, *const ComponentRef, usize) -> EntityId,
    despawn: unsafe extern "C" fn(*const c_void, EntityId),
    set_component_value: unsafe extern "C" fn(*const c_void, *const c_void, *const ComponentRef),
    load_asset: unsafe extern "C" fn(*const c_void, *const c_char) -> AssetId,
}

impl Default for Engine {
    fn default() -> Self {
        unreachable!("cannot construct Engine resource")
    }
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

impl Engine {
    /// Spawns an entity with the specified components.
    ///
    /// Returns the `EntityId` of the new entity.
    ///
    /// NOTE: spawns are deferred until the end of the frame, so the spawned entity will
    /// not be iterated by queries on the frame it is spawned.
    pub fn spawn(&self, components: &[ComponentRef]) -> EntityId {
        unsafe { (self.spawn)(self.engine_handle, components.as_ptr(), components.len()) }
    }

    /// Despawns an entity with the specified `EntityId`.
    ///
    /// NOTE: despawns are deferred until the end of the frame, so the despawned entity will
    /// still be iterated by queries on the frame it is despawned.
    pub fn despawn(&self, entity_id: EntityId) {
        unsafe {
            (self.despawn)(self.engine_handle, entity_id);
        }
    }

    /// Loads a static mesh asset. It is safe to call this for the same asset multiple times.
    ///
    /// Returns the identifying `AssetId` of the newly-loaded asset. If the asset has already been
    /// loaded, it will return the `AssetId` of the originally-loaded asset.
    pub fn load_asset(&self, asset_path: &CStr) -> AssetId {
        unsafe { (self.load_asset)(self.engine_handle, asset_path.as_ptr()) }
    }
}

/// A query is essentially an iterator over a number of entities, based on the specified
/// template components. For example, a query of type `Query<&Transform>` will iterate over
/// all the entities with a Transform component, and provide access to their `Transform` component.
///
/// Generic `Q` specifies the components to include in this query. Components *must* be references.
/// If the query specifies more than one component, `Q` should be a tuple (i.e. `Query<(&A, &B)>`).
#[repr(C)]
pub struct Query<Q> {
    query_handle: *mut c_void,
    marker: PhantomData<Q>,
}

unsafe impl<Q> Send for Query<Q> {}
unsafe impl<Q> Sync for Query<Q> {}

impl<Q> Query<Q> {
    pub fn new(query_handle: *mut c_void) -> Self {
        Self {
            query_handle,
            marker: PhantomData,
        }
    }

    /// Returns a reference to the requested component for an entity in this query.
    ///
    /// `entity_id` is the entity to get the component of. `T` must be one of the components in `Q`.
    ///
    /// Returns `None` if the lookup failed (i.e. the entity does not exist in this query).
    pub fn get<T: Component>(&self, entity_id: EntityId) -> Option<&T> {
        unsafe {
            let ptr =
                _QUERY_GET_FN.unwrap_unchecked()(self.query_handle, entity_id, T::id()) as *const T;
            ptr.as_ref()
        }
    }

    /// Returns a mutable reference to the requested component for an entity in this query.
    ///
    /// `entity_id` is the entity to get the component of. `T` must be one of the components in `Q`.
    ///
    /// Returns `None` if the lookup failed (i.e. the entity does not exist in this query).
    pub fn get_mut<T: Component>(&mut self, entity_id: EntityId) -> Option<&mut T> {
        unsafe {
            let ptr = _QUERY_GET_MUT_FN.unwrap_unchecked()(self.query_handle, entity_id, T::id())
                as *mut T;
            ptr.as_mut()
        }
    }

    /// Returns a reference to the requested component for the first entity in query. This can be
    /// useful when the query covers only a single entity.
    ///
    /// Returns `None` if the lookup failed (i.e. the query does not cover any entities).
    pub fn get_first<T: Component>(&self) -> Option<&T> {
        unsafe {
            let ptr =
                _QUERY_GET_FIRST_FN.unwrap_unchecked()(self.query_handle, T::id()) as *const T;
            ptr.as_ref()
        }
    }

    /// Returns a mutable reference to the requested component for the first entity in query. This
    /// can be useful when the query covers only a single entity.
    ///
    /// Returns `None` if the lookup failed (i.e. the query does not cover any entities).
    pub fn get_first_mut<T: Component>(&mut self) -> Option<&mut T> {
        unsafe {
            let ptr =
                _QUERY_GET_FIRST_MUT_FN.unwrap_unchecked()(self.query_handle, T::id()) as *mut T;
            ptr.as_mut()
        }
    }

    /// Iterates over all entities in this query by calling the provided function once per entity.
    ///
    /// This function only runs on a single thread. Prefer `par_for_each` where possible
    /// (see `par_for_each` docs for details).
    ///
    /// The parameters of the function *must* match the order and mutability of the query template.
    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(Q),
    {
        unsafe extern "C" fn callback<Q, F: FnMut(Q)>(
            entity_data: *mut *mut c_void,
            user_data: *mut c_void,
        ) -> c_int {
            match catch_unwind(|| {
                let f = &mut *(user_data as *mut F);
                f(transmute_copy(&*(entity_data as *mut Q)));
            }) {
                Ok(..) => 0,
                Err(..) => 1,
            }
        }

        unsafe {
            _QUERY_FOR_EACH_FN.unwrap_unchecked()(
                self.query_handle,
                callback::<Q, F>,
                &mut f as *mut _ as _,
            );
        }
    }

    /// Iterates over all entities in this query by calling the provided function once per entity.
    ///
    /// This version of for-each will be run in parallel and can provide significant performance improvements.
    /// This version should be the default, unless it is necessary to mutate captured state.
    ///
    /// The parameters of the function *must* match the order and mutability of the query template.
    pub fn par_for_each<F>(&mut self, f: F)
    where
        F: Fn(Q) + Send + Sync,
    {
        unsafe extern "C" fn callback<Q, F: Fn(Q)>(
            entity_data: *mut *mut c_void,
            user_data: *const c_void,
        ) -> c_int {
            match catch_unwind(|| {
                let f = &*(user_data as *const F);
                f(transmute_copy(&*(entity_data as *mut Q)));
            }) {
                Ok(..) => 0,
                Err(..) => 1,
            }
        }

        unsafe {
            _QUERY_PAR_FOR_EACH_FN.unwrap_unchecked()(
                self.query_handle,
                callback::<Q, F>,
                &f as *const _ as _,
            );
        }
    }
}

// global callback functions

pub static mut _QUERY_GET_FN: Option<
    unsafe extern "C" fn(*const c_void, EntityId, ComponentId) -> *const c_void,
> = None;

pub static mut _QUERY_GET_MUT_FN: Option<
    unsafe extern "C" fn(*mut c_void, EntityId, ComponentId) -> *mut c_void,
> = None;

pub static mut _QUERY_GET_FIRST_FN: Option<
    unsafe extern "C" fn(*const c_void, ComponentId) -> *const c_void,
> = None;

pub static mut _QUERY_GET_FIRST_MUT_FN: Option<
    unsafe extern "C" fn(*mut c_void, ComponentId) -> *mut c_void,
> = None;

pub static mut _QUERY_FOR_EACH_FN: Option<
    unsafe extern "C" fn(
        *mut c_void,
        unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> c_int,
        *mut c_void,
    ),
> = None;

pub static mut _QUERY_PAR_FOR_EACH_FN: Option<
    unsafe extern "C" fn(
        *mut c_void,
        unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> c_int,
        *const c_void,
    ),
> = None;
