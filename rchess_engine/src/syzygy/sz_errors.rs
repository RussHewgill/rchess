
pub use crate::syzygy::sz_format::*;

use std::fmt;
use std::io;
use std::error::Error;

pub type SyzygyResult<T> = Result<T, SyzygyError>;
pub type ProbeResult<T> = Result<T, ProbeError>;

/// Error when probing tablebase.
#[derive(Debug)]
pub enum SyzygyError {
    /// Position has castling rights, but Syzygy tables do not contain
    /// positions with castling rights.
    Castling,
    /// Position has too many pieces. Syzygy tables only support up to
    /// 6 or 7 pieces.
    TooManyPieces,
    /// Missing table.
    MissingTable {
        metric:   Metric,
        // material: Material
    },
    /// Probe failed.
    ProbeFailed {
        metric:   Metric,
        // material: Material,
        error:    ProbeError,
    },
}

impl fmt::Display for SyzygyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyzygyError::Castling =>
                write!(f, "syzygy tables do not contain position with castling rights"),
            SyzygyError::TooManyPieces =>
                write!(f, "too many pieces"),
            SyzygyError::MissingTable { metric } =>
                write!(f, "required {} table not found: (mat)", metric),
            SyzygyError::ProbeFailed { metric, error } =>
                write!(f, "failed to probe {} table (mat): {}", metric, error),
        }
    }
}

impl Error for SyzygyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SyzygyError::ProbeFailed { error, .. } => Some(error),
            _ => None,
        }
    }
}

/// Error when probing a table.
#[derive(Debug)]
pub enum ProbeError {
    /// I/O error.
    Read { error: io::Error },
    /// Table file has unexpected magic header bytes.
    Magic { magic: [u8; 4] },
    /// Corrupted table.
    CorruptedTable {
        #[cfg(feature = "backtrace")]
        backtrace: std::backtrace::Backtrace
    },
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeError::Read { error } =>
                write!(f, "i/o error reading table file: {}", error),
            ProbeError::Magic { magic } =>
                write!(f, "invalid magic header bytes: {:x?}", magic),
            ProbeError::CorruptedTable { .. } =>
                write!(f, "corrupted table"),
        }
    }
}

impl Error for ProbeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProbeError::Read { error } => Some(error),
            _ => None,
        }
    }

    #[cfg(feature = "backtrace")]
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        match self {
            ProbeError::CorruptedTable { backtrace } => Some(backtrace),
            _ => None
        }
    }
}

impl From<io::Error> for ProbeError {
    fn from(error: io::Error) -> ProbeError {
        match error.kind() {
            io::ErrorKind::UnexpectedEof => ProbeError::CorruptedTable {
                #[cfg(feature = "backtrace")]
                backtrace: std::backtrace::Backtrace::capture()
            },
            _ => ProbeError::Read { error },
        }
    }
}

/// Return a `CorruptedTable` error.
macro_rules! throw {
    () => {
        return Err(ProbeError::CorruptedTable {
            #[cfg(feature = "backtrace")]
            backtrace: ::std::backtrace::Backtrace::capture()
        })
    }
}

/// Unwrap an `Option` or return a `CorruptedTable` error.
macro_rules! u {
    ($e:expr) => {
        match $e {
            Some(ok) => ok,
            None => throw!(),
        }
    };
}

/// Ensure that a condition holds. Otherwise return a `CorruptedTable` error.
macro_rules! ensure {
    ($cond:expr) => {
        if !$cond {
            throw!();
        }
    };
}


