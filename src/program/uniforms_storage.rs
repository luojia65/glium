use std::cell::RefCell;
use RawUniformValue;

use smallvec::SmallVec;

use gl;
use Handle;
use context::CommandContext;
use version::Version;
use version::Api;

pub struct UniformsStorage {
    values: RefCell<SmallVec<[Option<RawUniformValue>; 16]>>,
    uniform_blocks: RefCell<SmallVec<[Option<gl::types::GLuint>; 4]>>,
    shader_storage_blocks: RefCell<SmallVec<[Option<gl::types::GLuint>; 4]>>,
}

impl UniformsStorage {
    /// Builds a new empty storage.
    #[inline]
    pub fn new() -> UniformsStorage {
        UniformsStorage {
            values: RefCell::new(SmallVec::new()),
            uniform_blocks: RefCell::new(SmallVec::new()),
            shader_storage_blocks: RefCell::new(SmallVec::new()),
        }
    }

    /// Compares `value` with the value stored in this object. If the values differ, updates
    /// the storage and calls `glUniform`.
    pub fn set_uniform_value(&self, ctxt: &mut CommandContext, program: Handle,
                             location: gl::types::GLint, value: &RawUniformValue)
    {
        let mut values = self.values.borrow_mut();

        if values.len() <= location as usize {
            for _ in (values.len() .. location as usize + 1) {
                values.push(None);
            }
        }

        // TODO: don't assume that, instead use DSA if the program is not current
        assert!(ctxt.state.program == program);

        macro_rules! uniform(
            ($ctxt:expr, $uniform:ident, $uniform_arb:ident, $($params:expr),+) => (
                unsafe {
                    if $ctxt.version >= &Version(Api::Gl, 1, 5) ||
                       $ctxt.version >= &Version(Api::GlEs, 2, 0)
                    {
                        $ctxt.gl.$uniform($($params),+)
                    } else {
                        assert!($ctxt.extensions.gl_arb_shader_objects);
                        $ctxt.gl.$uniform_arb($($params),+)
                    }
                }
            )
        );

        macro_rules! uniform64(
            ($ctxt:expr, $uniform:ident, $($params:expr),+) => (
                unsafe {
                    if $ctxt.extensions.gl_arb_gpu_shader_fp64 {
                        $ctxt.gl.$uniform($($params),+)
                    } else {
                        panic!("Double precision is not supported on this system.")
                    }
                }
            )
        );

        match (value, &mut values[location as usize]) {
            (&RawUniformValue::SignedInt(a), &mut Some(RawUniformValue::SignedInt(b))) if a == b => (),
            (&RawUniformValue::UnsignedInt(a), &mut Some(RawUniformValue::UnsignedInt(b))) if a == b => (),
            (&RawUniformValue::Float(a), &mut Some(RawUniformValue::Float(b))) if a == b => (),
            (&RawUniformValue::Mat2(a), &mut Some(RawUniformValue::Mat2(b))) if a == b => (),
            (&RawUniformValue::Mat3(a), &mut Some(RawUniformValue::Mat3(b))) if a == b => (),
            (&RawUniformValue::Mat4(a), &mut Some(RawUniformValue::Mat4(b))) if a == b => (),
            (&RawUniformValue::Vec2(a), &mut Some(RawUniformValue::Vec2(b))) if a == b => (),
            (&RawUniformValue::Vec3(a), &mut Some(RawUniformValue::Vec3(b))) if a == b => (),
            (&RawUniformValue::Vec4(a), &mut Some(RawUniformValue::Vec4(b))) if a == b => (),
            (&RawUniformValue::IntVec2(a), &mut Some(RawUniformValue::IntVec2(b))) if a == b => (),
            (&RawUniformValue::IntVec3(a), &mut Some(RawUniformValue::IntVec3(b))) if a == b => (),
            (&RawUniformValue::IntVec4(a), &mut Some(RawUniformValue::IntVec4(b))) if a == b => (),
            (&RawUniformValue::UnsignedIntVec2(a), &mut Some(RawUniformValue::UnsignedIntVec2(b))) if a == b => (),
            (&RawUniformValue::UnsignedIntVec3(a), &mut Some(RawUniformValue::UnsignedIntVec3(b))) if a == b => (),
            (&RawUniformValue::UnsignedIntVec4(a), &mut Some(RawUniformValue::UnsignedIntVec4(b))) if a == b => (),
            (&RawUniformValue::Double(a), &mut Some(RawUniformValue::Double(b))) if a == b => (),
            (&RawUniformValue::DoubleMat2(a), &mut Some(RawUniformValue::DoubleMat2(b))) if a == b => (),
            (&RawUniformValue::DoubleMat3(a), &mut Some(RawUniformValue::DoubleMat3(b))) if a == b => (),
            (&RawUniformValue::DoubleMat4(a), &mut Some(RawUniformValue::DoubleMat4(b))) if a == b => (),
            (&RawUniformValue::DoubleVec2(a), &mut Some(RawUniformValue::DoubleVec2(b))) if a == b => (),
            (&RawUniformValue::DoubleVec3(a), &mut Some(RawUniformValue::DoubleVec3(b))) if a == b => (),
            (&RawUniformValue::DoubleVec4(a), &mut Some(RawUniformValue::DoubleVec4(b))) if a == b => (),

            (&RawUniformValue::SignedInt(v), target) => {
                *target = Some(RawUniformValue::SignedInt(v));
                uniform!(ctxt, Uniform1i, Uniform1iARB, location, v);
            },

            (&RawUniformValue::UnsignedInt(v), target) => {
                *target = Some(RawUniformValue::UnsignedInt(v));

                // Uniform1uiARB doesn't exist
                unsafe {
                    if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                       ctxt.version >= &Version(Api::GlEs, 2, 0)
                    {
                        ctxt.gl.Uniform1ui(location, v)
                    } else {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.Uniform1iARB(location, v as gl::types::GLint)
                    }
                }
            },

            (&RawUniformValue::Float(v), target) => {
                *target = Some(RawUniformValue::Float(v));
                uniform!(ctxt, Uniform1f, Uniform1fARB, location, v);
            },

            (&RawUniformValue::Mat2(v), target) => {
                *target = Some(RawUniformValue::Mat2(v));
                uniform!(ctxt, UniformMatrix2fv, UniformMatrix2fvARB,
                         location, 1, gl::FALSE, v.as_ptr() as *const f32);
            },

            (&RawUniformValue::Mat3(v), target) => {
                *target = Some(RawUniformValue::Mat3(v));
                uniform!(ctxt, UniformMatrix3fv, UniformMatrix3fvARB,
                         location, 1, gl::FALSE, v.as_ptr() as *const f32);
            },

            (&RawUniformValue::Mat4(v), target) => {
                *target = Some(RawUniformValue::Mat4(v));
                uniform!(ctxt, UniformMatrix4fv, UniformMatrix4fvARB,
                         location, 1, gl::FALSE, v.as_ptr() as *const f32);
            },

            (&RawUniformValue::Vec2(v), target) => {
                *target = Some(RawUniformValue::Vec2(v));
                uniform!(ctxt, Uniform2fv, Uniform2fvARB, location, 1, v.as_ptr() as *const f32);
            },

            (&RawUniformValue::Vec3(v), target) => {
                *target = Some(RawUniformValue::Vec3(v));
                uniform!(ctxt, Uniform3fv, Uniform3fvARB, location, 1, v.as_ptr() as *const f32);
            },

            (&RawUniformValue::Vec4(v), target) => {
                *target = Some(RawUniformValue::Vec4(v));
                uniform!(ctxt, Uniform4fv, Uniform4fvARB, location, 1, v.as_ptr() as *const f32);
            },

            (&RawUniformValue::IntVec2(v), target) => {
                *target = Some(RawUniformValue::IntVec2(v));
                uniform!(ctxt, Uniform2iv, Uniform2ivARB, location, 1, v.as_ptr() as *const gl::types::GLint);
            },

            (&RawUniformValue::IntVec3(v), target) => {
                *target = Some(RawUniformValue::IntVec3(v));
                uniform!(ctxt, Uniform3iv, Uniform3ivARB, location, 1, v.as_ptr() as *const gl::types::GLint);
            },

            (&RawUniformValue::IntVec4(v), target) => {
                *target = Some(RawUniformValue::IntVec4(v));
                uniform!(ctxt, Uniform4iv, Uniform4ivARB, location, 1, v.as_ptr() as *const gl::types::GLint);
            },

            (&RawUniformValue::UnsignedIntVec2(v), target) => {
                *target = Some(RawUniformValue::UnsignedIntVec2(v));

                // Uniform2uivARB doesn't exist
                unsafe {
                    if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                       ctxt.version >= &Version(Api::GlEs, 2, 0)
                    {
                        ctxt.gl.Uniform2uiv(location, 1, v.as_ptr() as *const gl::types::GLuint)
                    } else {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.Uniform2ivARB(location, 1, v.as_ptr() as *const gl::types::GLint)
                    }
                }
            },

            (&RawUniformValue::UnsignedIntVec3(v), target) => {
                *target = Some(RawUniformValue::UnsignedIntVec3(v));

                // Uniform3uivARB doesn't exist
                unsafe {
                    if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                       ctxt.version >= &Version(Api::GlEs, 2, 0)
                    {
                        ctxt.gl.Uniform3uiv(location, 1, v.as_ptr() as *const gl::types::GLuint)
                    } else {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.Uniform3ivARB(location, 1, v.as_ptr() as *const gl::types::GLint)
                    }
                }
            },

            (&RawUniformValue::UnsignedIntVec4(v), target) => {
                *target = Some(RawUniformValue::UnsignedIntVec4(v));

                // Uniform4uivARB doesn't exist
                unsafe {
                    if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                       ctxt.version >= &Version(Api::GlEs, 2, 0)
                    {
                        ctxt.gl.Uniform4uiv(location, 1, v.as_ptr() as *const gl::types::GLuint)
                    } else {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.Uniform4ivARB(location, 1, v.as_ptr() as *const gl::types::GLint)
                    }
                }
            },
            (&RawUniformValue::Double(v), target) => {
                *target = Some(RawUniformValue::Double(v));
                uniform64!(ctxt, Uniform1d, location, v);
            },

            (&RawUniformValue::DoubleMat2(v), target) => {
                *target = Some(RawUniformValue::DoubleMat2(v));
                uniform64!(ctxt, UniformMatrix2dv,
                         location, 1, gl::FALSE, v.as_ptr() as *const gl::types::GLdouble);
            },

            (&RawUniformValue::DoubleMat3(v), target) => {
                *target = Some(RawUniformValue::DoubleMat3(v));
                uniform64!(ctxt, UniformMatrix3dv,
                         location, 1, gl::FALSE, v.as_ptr() as *const gl::types::GLdouble);
            },

            (&RawUniformValue::DoubleMat4(v), target) => {
                *target = Some(RawUniformValue::DoubleMat4(v));
                uniform64!(ctxt, UniformMatrix4dv,
                         location, 1, gl::FALSE, v.as_ptr() as *const gl::types::GLdouble);
            },

            (&RawUniformValue::DoubleVec2(v), target) => {
                *target = Some(RawUniformValue::DoubleVec2(v));
                uniform64!(ctxt, Uniform2dv, location, 1, v.as_ptr() as *const gl::types::GLdouble);
            },

            (&RawUniformValue::DoubleVec3(v), target) => {
                *target = Some(RawUniformValue::DoubleVec3(v));
                uniform64!(ctxt, Uniform3dv, location, 1, v.as_ptr() as *const gl::types::GLdouble);
            },

            (&RawUniformValue::DoubleVec4(v), target) => {
                *target = Some(RawUniformValue::DoubleVec4(v));
                uniform64!(ctxt, Uniform4dv, location, 1, v.as_ptr() as *const gl::types::GLdouble);
            },
        }
    }

    /// Compares `value` with the value stored in this object. If the values differ, updates
    /// the storage and calls `glUniformBlockBinding`.
    pub fn set_uniform_block_binding(&self, ctxt: &mut CommandContext, program: Handle,
                                     location: gl::types::GLuint, value: gl::types::GLuint)
    {
        let mut blocks = self.uniform_blocks.borrow_mut();

        if blocks.len() <= location as usize {
            for _ in (blocks.len() .. location as usize + 1) {
                blocks.push(None);
            }
        }

        // TODO: don't assume that, instead use DSA if the program is not current
        assert!(ctxt.state.program == program);

        match (value, &mut blocks[location as usize]) {
            (a, &mut Some(b)) if a == b => (),

            (a, target) => {
                *target = Some(a);
                match program {
                    Handle::Id(id) => unsafe {
                        ctxt.gl.UniformBlockBinding(id, location, value);
                    },
                    _ => unreachable!()
                }
            },
        }
    }

    /// Compares `value` with the value stored in this object. If the values differ, updates
    /// the storage and calls `glShaderStorageBlockBinding`.
    pub fn set_shader_storage_block_binding(&self, ctxt: &mut CommandContext, program: Handle,
                                            location: gl::types::GLuint, value: gl::types::GLuint)
    {
        let mut blocks = self.shader_storage_blocks.borrow_mut();

        if blocks.len() <= location as usize {
            for _ in (blocks.len() .. location as usize + 1) {
                blocks.push(None);
            }
        }

        // TODO: don't assume that, instead use DSA if the program is not current
        assert!(ctxt.state.program == program);

        match (value, &mut blocks[location as usize]) {
            (a, &mut Some(b)) if a == b => (),

            (a, target) => {
                *target = Some(a);
                match program {
                    Handle::Id(id) => unsafe {
                        ctxt.gl.ShaderStorageBlockBinding(id, location, value);
                    },
                    _ => unreachable!()
                }
            },
        }
    }
}
