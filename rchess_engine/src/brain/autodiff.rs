
// pub use self::dual_numbers::*;

pub mod dual_numbers {

    use num_traits::{Num,Pow,Float,NumCast};

    #[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
    pub struct Dual<T> {
        real: T,
        dual: T,
    }

    impl<T: Num> Dual<T> {
        pub fn new(real: T, dual: T) -> Self {
            Self { real, dual }
        }
        pub fn constant(real: T) -> Self {
            Self::new(real, T::zero())
        }
        pub fn variable(real: T) -> Self {
            Self::new(real, T::one())
        }
    }

    impl<T: Float> Dual<T> {
        #[inline]
        pub fn powi(self, n: i32) -> Self {
            let nf = <T as NumCast>::from(n).expect("Invalid value");
            Self::new(self.real.powi(n), nf * self.real.powi(n - 1) * self.dual)
        }
        #[inline]
        pub fn powf(self, other: Self) -> Self {
            let real: T = self.real.powf(other.real);
            let dual: T = other.real * self.real.powf(other.real - T::one()) * self.dual
                + real * self.real.ln() * other.dual;
            Self::new(real,dual)
        }
    }

    impl<T: Num> std::ops::Add<Dual<T>> for Dual<T> {
        type Output = Dual<T>;
        #[inline]
        fn add(self, other: Dual<T>) -> Self::Output {
            Self::Output::new(self.real + other.real, self.dual + other.dual)
        }
    }

    impl<T: Num> std::ops::Sub<Dual<T>> for Dual<T> {
        type Output = Dual<T>;
        #[inline]
        fn sub(self, other: Dual<T>) -> Self::Output {
            Self::Output::new(self.real - other.real, self.dual - other.dual)
        }
    }

    impl<T: Num + Copy> std::ops::Mul<Dual<T>> for Dual<T> {
        type Output = Dual<T>;
        #[inline]
        fn mul(self, other: Dual<T>) -> Self::Output {
            Self::Output::new(self.real * other.real, self.real * other.dual + self.dual * other.real)
        }
    }

    impl<T: Num> std::ops::Div<Dual<T>> for Dual<T> {
        type Output = Dual<T>;
        #[inline]
        fn div(self, other: Dual<T>) -> Self::Output {
            unimplemented!()
        }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for Dual<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&format!("Dual({:?} + {:?}e)", self.real, self.dual))?;
            Ok(())
        }
    }
}

use num_traits::{Num,Pow,Float,NumCast};

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub struct Dual<T, const N: usize> {
    val:   T,
    e:     [T; N],
}

pub fn differentiate<T: Num, F, const N: usize>(x: T, f: F) -> T
    where F: Fn(Dual<T,N>) -> Dual<T,N>,
{
    // f(Dual::new())
    unimplemented!()
}

impl<T: Num + Copy, const N: usize> Dual<T,N> {
    pub fn new(val: T, e: [T; N]) -> Self {
        Self { val, e }
    }
    pub fn constant(val: T) -> Self {
        Self {
            val,
            e:   [T::zero(); N],
        }
    }
    pub fn variable(val: T, index: usize) -> Self {
        let mut e = [T::zero(); N];
        e[index] = T::one();
        Self {
            val,
            e,
        }
    }
}

