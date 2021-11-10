
use nalgebra::{SMatrix,SVector,Matrix,Vector,matrix,vector,Dynamic,VecStorage};
use nalgebra::Dynamic as Dy;

use ndarray::ShapeBuilder;

// use nshare::{RefNdarray1,RefNdarray2,ToNdarray1,ToNdarray2,ToNalgebra};

pub trait RefNdarray2 {
    type Out;
    fn ref_ndarray2(self) -> Self::Out;
}

pub trait ToNalgebra {
    type Out;
    fn into_nalgebra(self) -> Self::Out;
}

impl<'a, N: nalgebra::Scalar, R: nalgebra::Dim, C: nalgebra::Dim, S> RefNdarray2 for &'a Matrix<N, R, C, S>
where
    S: nalgebra::Storage<N, R, C>,
{
    type Out = ndarray::ArrayView2<'a, N>;

    fn ref_ndarray2(self) -> Self::Out {
        unsafe { ndarray::ArrayView2::from_shape_ptr(self.shape().strides(self.strides()), self.as_ptr()) }
    }
}

impl<T> ToNalgebra for ndarray::Array1<T>
where
    T: nalgebra::Scalar,
{
    type Out = nalgebra::DVector<T>;
    fn into_nalgebra(self) -> Self::Out {
        let len = Dy::new(self.len());
        Self::Out::from_vec_generic(len, nalgebra::Const::<1>, self.into_raw_vec())
    }
}

impl<T> ToNalgebra for ndarray::Array2<T>
where
    T: nalgebra::Scalar,
{
    type Out = nalgebra::DMatrix<T>;
    fn into_nalgebra(self) -> Self::Out {
        let std_layout = self.is_standard_layout();
        let nrows = Dy::new(self.nrows());
        let ncols = Dy::new(self.ncols());
        let mut res = Self::Out::from_vec_generic(nrows, ncols, self.into_raw_vec());
        if std_layout {
            // This can be expensive, but we have no choice since nalgebra VecStorage is always
            // column-based.
            res.transpose_mut();
        }
        res
    }
}

