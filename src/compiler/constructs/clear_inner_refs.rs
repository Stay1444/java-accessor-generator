use crate::{java, Object};

const START_BODY: &str = r#"
this.self = null;
"#;

const CLEAR_SINGLE_REF: &str = r#"
if (this.{{FIELD}} != null) this.{{FIELD}}.clearInnerRefs();
"#;

const CLEAR_ARRAY_REF: &str = r#"
clearInnerRefsArray(this.{{FIELD}});
"#;

fn field_is_enum(current: &Object, field: &crate::Field, includes: &[Object]) -> bool {
    if let Some(name) = field.field_type.ends_in_object(current) {
        if includes.iter().any(|x| x.name == name) {
            return true;
        }
    }

    false
}

pub fn generate(object: &Object, includes: &[Object]) -> java::Method {
    java::Method::builder()
        .name("clearInnerRefs")
        .body(
            vec![START_BODY.to_owned()]
                .into_iter()
                .chain(
                    object
                        .fields
                        .iter()
                        .filter(|field| {
                            !field.field_type.is_primitive()
                                && !field_is_enum(object, field, includes)
                        })
                        .map(|field| {
                            if field.field_type.is_nested_array()
                                || matches!(field.field_type, crate::Type::Array(_))
                            {
                                CLEAR_ARRAY_REF
                            } else {
                                CLEAR_SINGLE_REF
                            }
                            .replace("{{FIELD}}", field.display_name())
                        })
                        .collect::<Vec<String>>(),
                )
                .collect::<Vec<String>>()
                .join("\n"),
        )
        .build()
}
