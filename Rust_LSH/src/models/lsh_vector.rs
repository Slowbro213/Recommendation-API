use::serde::{Deserialize, Serialize};

##[derive(Debug, Serialize, Deserialize)]
pub struct LSHVector {
    pub vector: Vec<f32>,
    pub id: String,
}
