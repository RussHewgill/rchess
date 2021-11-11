
use std::convert::TryFrom;

use nalgebra::{DMatrix, DVector, Dim, Dynamic, Matrix,
               SMatrix, SVector, Scalar, VecStorage, Vector, matrix, vector};
use nalgebra::Dynamic as Dy;

use ndarray::ShapeBuilder;

// use nshare::{RefNdarray1,RefNdarray2,ToNdarray1,ToNdarray2,ToNalgebra};


pub fn mul_blas(a: DMatrix<f32>, b: DMatrix<f32>) -> DMatrix<f32> {
    let aa = a.ref_ndarray2();
    let bb = b.ref_ndarray2();
    let out = aa.dot(&bb);
    out.into_nalgebra()
}


pub trait RefNdarray2 {
    type Out;
    fn ref_ndarray2(self) -> Self::Out;
}

pub trait ToNalgebra {
    type Out;
    fn into_nalgebra(self) -> Self::Out;
}

impl<'a, N: Scalar, R: Dim, C: Dim, S> RefNdarray2 for &'a Matrix<N, R, C, S>
where
    S: nalgebra::Storage<N, R, C>,
{
    type Out = ndarray::ArrayView2<'a, N>;

    fn ref_ndarray2(self) -> Self::Out {
        unsafe {
            ndarray::ArrayView2::from_shape_ptr(self.shape().strides(self.strides()), self.as_ptr())
        }
    }
}

impl<T> ToNalgebra for ndarray::Array1<T>
where
    T: Scalar,
{
    type Out = nalgebra::DVector<T>;
    fn into_nalgebra(self) -> Self::Out {
        let len = Dy::new(self.len());
        Self::Out::from_vec_generic(len, nalgebra::Const::<1>, self.into_raw_vec())
    }
}

impl<T> ToNalgebra for ndarray::Array2<T>
where
    T: Scalar,
{
    type Out = nalgebra::DMatrix<T>;
    fn into_nalgebra(self) -> Self::Out {
        let std_layout = self.is_standard_layout();
        let nrows = Dy::new(self.nrows());
        let ncols = Dy::new(self.ncols());
        let mut res = Self::Out::from_vec_generic(nrows, ncols, self.into_raw_vec());
        if !std_layout {
            // This can be expensive, but we have no choice since nalgebra VecStorage is always
            // column-based.
            // res.transpose_mut();
            res = res.transpose();
        }
        res
    }
}

impl<'a, T> ToNalgebra for ndarray::ArrayView1<'a, T>
where
    T: Scalar,
{
    type Out = nalgebra::DVectorSlice<'a, T>;
    fn into_nalgebra(self) -> Self::Out {
        let len = Dy::new(self.len());
        let ptr = self.as_ptr();
        let stride: usize = TryFrom::try_from(self.strides()[0]).expect("Negative stride");
        let storage = unsafe {
            nalgebra::SliceStorage::from_raw_parts(
                ptr,
                (len, nalgebra::Const::<1>),
                (nalgebra::Const::<1>, Dy::new(stride)),
            )
        };
        nalgebra::Matrix::from_data(storage)
    }
}

impl<'a, T> ToNalgebra for ndarray::ArrayViewMut1<'a, T>
where
    T: nalgebra::Scalar,
{
    type Out = nalgebra::DVectorSliceMut<'a, T>;
    fn into_nalgebra(mut self) -> Self::Out {
        let len = Dy::new(self.len());
        let stride: usize = TryFrom::try_from(self.strides()[0]).expect("Negative stride");
        let ptr = self.as_mut_ptr();
        let storage = unsafe {
            // Drop to not have simultaneously the ndarray and nalgebra valid.
            drop(self);
            nalgebra::SliceStorageMut::from_raw_parts(
                ptr,
                (len, nalgebra::Const::<1>),
                (nalgebra::Const::<1>, Dy::new(stride)),
            )
        };
        nalgebra::Matrix::from_data(storage)
    }
}

impl<'a, T> ToNalgebra for ndarray::ArrayView2<'a, T>
where
    T: nalgebra::Scalar,
{
    type Out = nalgebra::DMatrixSlice<'a, T, Dy, Dy>;
    fn into_nalgebra(self) -> Self::Out {
        let nrows = Dy::new(self.nrows());
        let ncols = Dy::new(self.ncols());
        let ptr = self.as_ptr();
        let stride_row: usize = TryFrom::try_from(self.strides()[0]).expect("Negative row stride");
        let stride_col: usize =
            TryFrom::try_from(self.strides()[1]).expect("Negative column stride");
        let storage = unsafe {
            nalgebra::SliceStorage::from_raw_parts(
                ptr,
                (nrows, ncols),
                (Dy::new(stride_row), Dy::new(stride_col)),
            )
        };
        nalgebra::Matrix::from_data(storage)
    }
}

impl<'a, T> ToNalgebra for ndarray::ArrayViewMut2<'a, T>
where
    T: nalgebra::Scalar,
{
    type Out = nalgebra::DMatrixSliceMut<'a, T, Dy, Dy>;
    fn into_nalgebra(mut self) -> Self::Out {
        let nrows = Dy::new(self.nrows());
        let ncols = Dy::new(self.ncols());
        let stride_row: usize = TryFrom::try_from(self.strides()[0]).expect("Negative row stride");
        let stride_col: usize =
            TryFrom::try_from(self.strides()[1]).expect("Negative column stride");
        let ptr = self.as_mut_ptr();
        let storage = unsafe {
            // Drop to not have simultaneously the ndarray and nalgebra valid.
            drop(self);
            nalgebra::SliceStorageMut::from_raw_parts(
                ptr,
                (nrows, ncols),
                (Dy::new(stride_row), Dy::new(stride_col)),
            )
        };
        nalgebra::Matrix::from_data(storage)
    }
}

