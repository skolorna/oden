pub const HUGGINGFACE_MODEL: &str = "amcoff/bert-based-swedish-cased-ner";

pub fn gen_search_query(pipeline: &trast::Pipeline, menu: &str) -> Result<String, trast::Error> {
    Ok(pipeline
        .predict(menu)?
        .into_iter()
        .map(|e| e.word)
        .collect::<Vec<_>>()
        .join(" "))
}
