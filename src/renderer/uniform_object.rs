#[repr(C)]
pub struct UniformObject {
    pub(crate) model: cgmath::Matrix4<f32>,
    pub(crate) view: cgmath::Matrix4<f32>,
    pub(crate) projection: cgmath::Matrix4<f32>,
}
