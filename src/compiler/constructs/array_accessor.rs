use crate::{java, Object};

const BODY: &str = r#"
if (object == null) return null;

var entries = new java.util.ArrayList<>();

for (var entry : object) {
    entries.add({{NAME}}Accessor.access(entry));
}

return entries.toArray(new {{NAME}}Accessor[0]);
"#;

pub fn generate(object: &Object) -> java::Method {
    java::Method::builder()
        .is_static(true)
        .exceptions(
            vec![
                "NoSuchFieldException",
                "SecurityException",
                "IllegalArgumentException",
                "IllegalAccessException",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        )
        .name("accessArray")
        .arguments(vec![java::Argument::builder()
            .name("object")
            .type_name("Object[]")
            .build()])
        .return_type(format!("{}Accessor[]", object.display_name()))
        .body(BODY.replace("{{NAME}}", object.display_name()))
        .build()
}
