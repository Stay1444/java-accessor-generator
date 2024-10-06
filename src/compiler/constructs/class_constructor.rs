use crate::{java, Object};

const BODY: &str = r#"
this.self = self;
"#;

pub fn generate(object: &Object) -> java::Method {
    java::Method::builder()
        .is_constructor(true)
        .name(format!("{}Accessor", object.display_name()))
        .arguments(vec![java::Argument::builder()
            .name("self")
            .type_name("Object")
            .build()])
        .body(BODY)
        .build()
}
