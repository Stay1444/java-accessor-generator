use crate::{java, Object, Type};

const START_BODY: &str = r#"
if (object == null) return null;

var accessor = new {{CLASS_NAME}}Accessor(object);
var clazz = object.getClass();
if (clazz.isEnum()) throw new RuntimeException("Failed to access {{CLASS_NAME}}: Expected object to be object but got enum.");
"#;

const FIELD_BODY: &str = r#"
{
    var field = clazz.{{FIELD_GET_METHOD}}("{{TRUE_FIELD_NAME}}");
    field.setAccessible(true);
    var value = field.{{GET_METHOD}}(object);
    accessor.{{FIELD_NAME}} = {{ACCESSOR}};
}
"#;

const END_BODY: &str = r#"
return accessor;
"#;

pub fn generate(object: &Object) -> java::Method {
    java::Method::builder()
        .is_static(true)
        .name("access")
        .return_type(format!("{}Accessor", object.display_name()))
        .arguments(vec![java::Argument::builder()
            .name("object")
            .type_name("Object")
            .build()])
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
        .body(
            vec![START_BODY
                .replace("{{CLASS_NAME}}", object.display_name())
                .to_owned()]
            .into_iter()
            .chain(
                object
                    .fields
                    .iter()
                    .map(|field| {
                        FIELD_BODY
                            .replace(
                                "{{FIELD_GET_METHOD}}",
                                if field.hierarchy {
                                    "getField"
                                } else {
                                    "getDeclaredField"
                                },
                            )
                            .replace("{{TRUE_FIELD_NAME}}", &field.name)
                            .replace(
                                "{{GET_METHOD}}",
                                match &field.field_type {
                                    Type::Bool => "getBoolean",
                                    Type::I32 => "getInt",
                                    Type::I64 => "getLong",
                                    Type::U8 => "getByte",
                                    Type::I16 => "getShort",
                                    Type::F32 => "getFloat",
                                    Type::F64 => "getDouble",
                                    Type::String
                                    | Type::Array(_)
                                    | Type::Object(_)
                                    | Type::SelfObject => "get",
                                },
                            )
                            .replace("{{FIELD_NAME}}", field.display_name())
                            .replace(
                                "{{ACCESSOR}}",
                                &field.field_type.generate_accessor(true, object),
                            )
                            .to_string()
                    })
                    .collect::<Vec<String>>(),
            )
            .chain(vec![END_BODY.to_string()])
            .collect::<Vec<String>>()
            .join("\n"),
        )
        .build()
}
