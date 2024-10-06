use bon::Builder;

use super::Visibility;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Field {
    pub name: String,
    pub type_name: String,
    pub comment: Option<String>,
    #[builder(default)]
    pub visibility: Visibility,
}
