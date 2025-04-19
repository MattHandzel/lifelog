
use definitions::*;
use thiserror::Error;
use std::fmt;

#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Error in transform {transform:?}: {message}")]
    General {
        transform: Transform,
        message: String,
    },
    #[error("Invalid input type for transform {transform:?}")]
    InvalidInputType { transform: Transform },
    #[error("Unknown error occurred")]
    Unknown,
}



enum TextTransformationTypes {
    TextEmbedding,
    EntityExtraction,
    KeywordExtraction,
}

enum ImageTransformationTypes {
    OCR,
    ImageEmbedding,
    SensitiveContentDetection,
}

enum TransformTypes {
    TextEmbedding,
    EntityExtraction,
    OCR,
    ImageEmbedding,
    SensitiveContentDetection,
}

struct TransformResult {
    schema: DatabaseSchema,
}

// F - from type, T - to type
trait Transform<F: DataType, T:DataType, C: TranformConfig> {
    // Takes in the input data and outpust the new data type
    fn apply(&self, input: F) -> Result<T, TransformError>;

    fn new(config: C) -> Self;

    // Returns the name
    fn name(&self) -> &'static str;

    // Returns the priority of the transform
    fn priority(&self) -> u8;
}

struct TransformGraphEdge {
    from: TransformGraphNode,
    to : TransformGraphNode,
    transform: Transform,
}

struct TransformGraphNode {
    transform: TransformResult,
    edges : Vec<TransformGraphEdge>,
}

struct TransformGraph {
    nodes: Vec<TransformGraphNode>,
    edges: Vec<TransformGraphEdge>,
}

struct TransformPipeline {
    transforms: Vec<Box<dyn Transform>>,
}

impl TransformPipeline<F: DataType, T:DataType, C: TranformConfig> {
    fn run(&self, Data) -> Result<Data, String> {
        for transform in &self.transforms {
            data = transform.apply(&data)?;
        }
        Ok(data)
    }
}
