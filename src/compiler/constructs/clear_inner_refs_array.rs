use crate::java;

const BODY: &str = r#"
if (obj == null) return;
if (obj.getClass().isArray()) {
    var array = (Object[])obj;

    for (var entry : array) {
        clearInnerRefsArray(entry);
    }
} else {
    try {
        var method = obj.getClass().getMethod("clearInnerRefs");
        method.invoke(obj);
    } catch (Exception e) {
        System.out.println("Error in clearRefsArray:");
        e.printStackTrace();
    }
}
"#;

pub fn generate() -> java::Method {
    java::Method::builder()
        .name("clearInnerRefsArray")
        .is_static(true)
        .visibility(java::Visibility::Private)
        .arguments(vec![java::Argument::builder()
            .name("obj")
            .type_name("Object")
            .build()])
        .body(BODY)
        .build()
}
