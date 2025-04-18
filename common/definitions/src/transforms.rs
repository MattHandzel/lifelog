use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};

use rust_bert::pipelines::ner::{
    Entity
};


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

struct <A, B> Transform {
    input_type: A,
    output_type: B,
}
trait Transform {
    apply(&self, data: Data) -> Result<Data, String>;
}

struct TransformResult {
    schema: DatabaseSchema,

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

impl TransformPipeline {
    fn run(&self, Data) -> Result<Data, String> {
        for transform in &self.transforms {
            data = transform.apply(&data)?;
        }
        Ok(data)
    }
}
