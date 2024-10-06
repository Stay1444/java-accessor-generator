use bon::Builder;

use super::Visibility;

#[derive(Builder, Debug)]
#[builder(on(String, into))]
pub struct Method {
    pub name: String,
    #[builder(default)]
    pub arguments: Vec<Argument>,
    pub return_type: Option<String>,
    #[builder(default)]
    pub is_constructor: bool,
    #[builder(default)]
    pub visibility: Visibility,
    #[builder(default)]
    pub is_static: bool,
    #[builder(default)]
    pub exceptions: Vec<String>,
    pub body: String,
}

#[derive(Builder, Debug)]
#[builder(on(String, into))]
pub struct Argument {
    pub name: String,
    pub type_name: String,
}
