use crate::{java, Object};

const BODY: &str = r#"
try {
    var clazz = this.self.getClass();
    var field = clazz.{{GET_METHOD}}("{{TRUE_FIELD_NAME}}");
    field.setAccessible(true);
    field.set(this.self, value);

    this.{{FIELD_NAME}} = value;
} catch (Exception e) {
    System.out.println("Error setting field '{{FIELD_NAME}}':");
    e.printStackTrace();
}
"#;

pub fn generate(fields: &[crate::Field], object: &Object) -> Vec<java::Method> {
    fields
        .iter()
        .map(|field| {
            java::Method::builder()
                .name(format!("set_{}", field.display_name()))
                .arguments(vec![java::Argument::builder()
                    .name("value")
                    .type_name(field.field_type.java_name(object))
                    .build()])
                .body(
                    BODY.replace(
                        "{{GET_METHOD}}",
                        if field.hierarchy {
                            "getField"
                        } else {
                            "getDeclaredField"
                        },
                    )
                    .replace("{{TRUE_FIELD_NAME}}", &field.name)
                    .replace("{{FIELD_NAME}}", field.display_name()),
                )
                .build()
        })
        .collect()
}
