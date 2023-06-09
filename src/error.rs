use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;

pub type GraphResult<T> = Result<T, GraphError>;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("The edge between {from:?} and {to:?} already exists.")]
    EdgeAlreadyExists { from: usize, to: usize },
    #[error("Two sided edge forbidden between {from:?} and {to:?} in directed graph.")]
    TwoSidedEdgeForbidden { from: usize, to: usize },
    #[error("The given edge list has a bad format")]
    BadEdgeListFormat,
    #[error("ParseIntError: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("ParseFloatError: {0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Could not find tsp solution with nearest neighbor")]
    NNAbort,
    #[error("Not all nodes have been visited")]
    NoCycle,
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    #[error("Minimal cost flow not solvable")]
    McfNotSolvable,
}
