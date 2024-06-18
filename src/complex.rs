use std::ops::Mul;
use std::ops::Add;


#[derive(Clone, Copy)]
pub struct Complex {
    pub r: f64,
    pub i: f64,
}

impl Complex {
    pub fn norm(self) -> f64 {
        (self.r * self.r + self.i * self.i).sqrt()
    }
}

impl Add for Complex {
    type Output = Complex;

    fn add(self, rhs: Self) -> Self::Output {
        Complex{
            i: self.i + rhs.i,
            r: self.r + rhs.r,
        }
    }
}

impl Mul for Complex {
    type Output = Complex;

    fn mul(self, rhs: Self) -> Self::Output {
        Complex {
            r: self.r * rhs.r - self.i * rhs.i,
            i: self.r * rhs.i + self.i * rhs.r,
        }
    }
}
