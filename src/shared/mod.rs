pub type ResultErr<DATA> = Result<DATA, TrainingError>;


#[derive(Debug)]
pub enum TrainingError {
    Simple(String),
}