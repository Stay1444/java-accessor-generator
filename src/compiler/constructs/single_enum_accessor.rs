use crate::{java, Object};

const START_BODY: &str = r#"
if (object == null) return null;

var clazz = object.getClass();

if (!clazz.isEnum()) throw new RuntimeException("{{NAME}} was supposed to be an enum but it is not!");

var variant = ((Enum<?>) object).name();

switch (variant) {
"#;

const END_BODY: &str = r#"
    default:
        var variants = clazz.getEnumConstants();
        System.out.println("{{NAME}} variants:");

        for (var c : variants) {
            var constant = (Enum<?>)c;
            System.out.println("    - '" + constant.name() + "': '" + constant.toString() + "'");
        }

        throw new RuntimeException("{{NAME}}Accessor has an unrecognized variant: '" + variant + "'");
}
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
            vec!["SecurityException", "IllegalArgumentException"]
                .into_iter()
                .map(String::from)
                .collect(),
        )
        .body(
            vec![START_BODY
                .replace("{{NAME}}", object.display_name())
                .to_owned()]
            .into_iter()
            .chain(
                object
                    .variants
                    .iter()
                    .map(|variant| {
                        format!(
                            "\tcase \"{}\": return {}Accessor.{};",
                            variant.name,
                            object.display_name(),
                            variant.display_name()
                        )
                    })
                    .collect::<Vec<String>>(),
            )
            .chain(vec![END_BODY
                .replace("{{NAME}}", object.display_name())
                .to_string()])
            .collect::<Vec<String>>()
            .join("\n"),
        )
        .build()
}
