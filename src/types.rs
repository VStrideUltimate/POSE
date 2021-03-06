use std::ops;
use std::iter::Sum;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Array3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Array3d {
    pub fn dot(&self, other: &Array3d) -> f64 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }
}

impl<'a> Sum<&'a Self> for Array3d {
    fn sum<I>(iter: I) -> Self 
    where I: Iterator<Item = &'a Self> { 
        iter.fold(Self {x: 0.0, y: 0.0, z: 0.0}, |acc, x| Self {
            x: acc.x + x.x,
            y: acc.y + x.y,
            z: acc.z + x.z
        })
    }
}

impl_op_ex!(+ |a: &Array3d, b: &Array3d| -> Array3d {
    Array3d {
        x: a.x + b.x,
        y: a.y + b.y,
        z: a.z + b.z,
    }
});

impl_op_ex_commutative!(+ |a: &Array3d, b: &f64| -> Array3d {
    Array3d {
        x: a.x + b,
        y: a.y + b,
        z: a.z + b,
    }
});

impl_op_ex!(- |a: &Array3d, b: &Array3d| -> Array3d {
    Array3d {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
});

impl_op_ex!(- |a: &Array3d, b: &f64| -> Array3d {
    Array3d {
        x: a.x - b,
        y: a.y - b,
        z: a.z - b,
    }
});

impl_op_ex!(* |a: &Array3d, b: &Array3d| -> Array3d {
    Array3d {
        x: a.x * b.x,
        y: a.y * b.y,
        z: a.z * b.z,
    }
});

impl_op_ex_commutative!(* |a: &Array3d, b: &f64| -> Array3d {
    Array3d {
        x: a.x * b,
        y: a.y * b,
        z: a.z * b,
    }
});