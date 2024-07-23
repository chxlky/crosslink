use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Device {
    pub ip: String,
    pub hostname: String,
}
