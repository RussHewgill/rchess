

use num_traits::Num;


#[derive(Clone,Copy)]
pub struct Dual<T> {
    real: T,
    dual: T,
}

impl<T: Num> Dual<T> {
    pub fn new(real: T, dual: T) -> Self {
        Self { real, dual }
    }
}

impl<T: Num> std::ops::Add<Dual<T>> for Dual<T> {
    type Output = Dual<T>;
    #[inline]
    fn add(self, other: Dual<T>) -> Self::Output {
        Self::Output::new(self.real + other.real, self.dual + other.dual)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Dual<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Dual({:?} + {:?}e)", self.real, self.dual))?;
        Ok(())
    }
}
