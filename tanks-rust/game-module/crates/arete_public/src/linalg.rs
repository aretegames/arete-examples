use std::{
    mem::transmute,
    ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use nalgebra_glm as glm;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    _padding: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            _padding: 0.0,
        }
    }

    pub fn x() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    pub fn y() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub fn z() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    pub fn cross(self, rhs: Self) -> Self {
        let lhs = glm::Vec3::new(self.x, self.y, self.z);
        let rhs = glm::Vec3::new(rhs.x, rhs.y, rhs.z);
        let val = lhs.cross(&rhs);
        Self::new(val.x, val.y, val.z)
    }

    pub fn dot(self, rhs: Self) -> f32 {
        let lhs = glm::Vec3::new(self.x, self.y, self.z);
        let rhs = glm::Vec3::new(rhs.x, rhs.y, rhs.z);
        lhs.dot(&rhs)
    }

    pub fn max(self) -> f32 {
        glm::Vec3::new(self.x, self.y, self.z).max()
    }

    pub fn norm(self) -> f32 {
        glm::Vec4::new(self.x, self.y, self.z, 0.0).norm()
    }

    pub fn norm_squared(self) -> f32 {
        glm::Vec4::new(self.x, self.y, self.z, 0.0).norm_squared()
    }

    pub fn try_normalize(self, min_norm: f32) -> Option<Self> {
        glm::Vec4::new(self.x, self.y, self.z, 0.0)
            .try_normalize(min_norm)
            .map(|val| Self::new(val.x, val.y, val.z))
    }

    pub fn normalize(self) -> Self {
        let val = glm::Vec4::new(self.x, self.y, self.z, 0.0).normalize();
        Self::new(val.x, val.y, val.z)
    }

    pub fn normalize_mut(&mut self) -> f32 {
        self._padding = 0.0;

        let val: &mut glm::Vec4 = unsafe { transmute(self) };
        val.normalize_mut()
    }

    pub fn to_homogeneous(self) -> glm::Vec4 {
        glm::Vec4::new(self.x, self.y, self.z, 0.0)
    }
}

impl From<glm::Vec3> for Vec3 {
    fn from(value: glm::Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<Vec3> for glm::Vec3 {
    fn from(value: Vec3) -> glm::Vec3 {
        glm::Vec3::new(value.x, value.y, value.z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            _padding: self._padding + rhs._padding,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self._padding += rhs._padding;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            _padding: self._padding - rhs._padding,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self._padding -= rhs._padding;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            _padding: self._padding * rhs,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
            _padding: rhs._padding * self,
        }
    }
}

impl Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            _padding: self._padding * rhs._padding,
        }
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self._padding *= rhs;
    }
}

impl MulAssign<Vec3> for Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self._padding *= rhs._padding;
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            _padding: self._padding / rhs,
        }
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self._padding /= rhs;
    }
}

impl DivAssign<Vec3> for Vec3 {
    fn div_assign(&mut self, rhs: Vec3) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
        self._padding /= rhs._padding;
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            _padding: -self._padding,
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct Quat(glm::Quat);

impl Default for Quat {
    fn default() -> Self {
        Self(glm::Quat::identity())
    }
}

impl From<glm::Quat> for Quat {
    fn from(value: glm::Quat) -> Self {
        Self(value)
    }
}

impl Deref for Quat {
    type Target = glm::Quat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Quat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
