use lifelog_core::*;
use lifelog_types::DataSource;
use std::fmt;
use thiserror::Error;

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

enum TransformType {
    TextEmbedding,
    EntityExtraction,
    OCR,
    ImageEmbedding,
    SensitiveContentDetection,
}

struct TransformConfig {}

struct TransformExampleStruct {
    input: DataSource,
    output: DataSource,
    config: TransformConfig,
}

pub trait Transform {
    type Input;
    type Output;
    type Config;

    fn apply(&self, input: Self::Input) -> Result<Self::Output, TransformError>;
    fn modality(&self) -> String;
    fn new(config: Self::Config) -> Self;

    fn priority(&self) -> u8;
}

//struct TransformGraphEdge {
//    from: TransformGraphNode,
//    to : TransformGraphNode,
//    transform: Transform,
//}
//
//struct TransformGraphNode<T> {
//    transform: ,
//    edges : Vec<TransformGraphEdge>,
//}
//
//struct TransformGraph {
//    nodes: Vec<TransformGraphNode>,
//    edges: Vec<TransformGraphEdge>,
//}
//
//struct TransformPipeline {
//    transforms: Vec<Box<dyn Transform>>,
//}
//
//struct TransformPipeline<F> {
//    transforms: Vec<Box<dyn Transform<F, T>>>,
//}
//
//impl TransformPipeline<F> {
//    fn new(config: TransformConfig) -> Self {
//        let mut transforms = Vec::new();
//        for transform in config.transforms {
//            let transform_instance = transform.new(config);
//            transforms.push(Box::new(transform_instance));
//        }
//        Self { transforms }
//    }
//    fn run(&self, F, db : surrealdb::) -> Result<T> {
//        Ok(data)
//    }
//}
