use crate::java;

const BODY: &str = r#"
if (object == null) return null;

if (!(object instanceof Object[]))
    return object;

var objArray = (Object[]) object;

var entries = new java.util.ArrayList<>();

for (var entry : objArray) {{
    entries.add(accessArrayNested(entry));
}}

return entries.toArray();
"#;

pub fn generate() -> java::Method {
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
        .name("accessArrayNested")
        .arguments(vec![java::Argument::builder()
            .name("object")
            .type_name("Object")
            .build()])
        .return_type("Object")
        .body(BODY)
        .build()
}
