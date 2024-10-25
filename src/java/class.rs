use bon::Builder;

use super::{Field, Method};

#[derive(Builder, Debug)]
#[builder(on(String, into))]
pub struct Class {
    pub name: String,
    #[builder(default)]
    pub class_type: ClassType,
    #[builder(default)]
    pub is_virtual: bool,
    pub package: String,
    pub comment: Option<String>,
    #[builder(default)]
    pub fields: Vec<Field>,
    #[builder(default)]
    pub methods: Vec<Method>,
    #[builder(default)]
    pub includes: Vec<String>,
}

#[allow(unused)]
#[derive(Default, Debug)]
pub enum ClassType {
    #[default]
    Class,
    Interface,
}

impl Class {
    pub fn source(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        writeln!(output, "package {};", self.package).unwrap();

        if !self.includes.is_empty() {
            writeln!(output).unwrap();
            for include in &self.includes {
                writeln!(output, "import {};", include).unwrap();
            }
        }

        if let Some(comment) = &self.comment {
            writeln!(output).unwrap();
            let lines: Vec<String> = comment.lines().map(|x| format!("// {x}")).collect();
            write!(output, "{}", lines.join("\n")).unwrap();
        }

        write!(output, "\npublic ").unwrap();
        if self.is_virtual {
            write!(output, "virtual ").unwrap();
        }
        match self.class_type {
            ClassType::Class => write!(output, "class ").unwrap(),
            ClassType::Interface => write!(output, "interface ").unwrap(),
        }
        writeln!(output, "{} {{", self.name).unwrap();

        for field in &self.fields {
            if let Some(comment) = &field.comment {
                let lines: Vec<String> = comment.lines().map(|x| format!("\t// {x}\n")).collect();
                write!(output, "{}", lines.join("\n")).unwrap();
            }
            writeln!(
                output,
                "\t{} {} {};",
                field.visibility,
                field.type_name,
                field.name
            )
            .unwrap();
        }

        if !self.methods.is_empty() {
            writeln!(output).unwrap();

            for method in &self.methods {
                write!(output, "\n\t{}", method.visibility).unwrap();
                if method.is_static {
                    write!(output, " static").unwrap();
                }

                if !method.is_constructor {
                    write!(
                        output,
                        " {}",
                        method.return_type.as_ref().unwrap_or(&String::from("void"))
                    )
                    .unwrap();
                }

                write!(output, " {}(", method.name).unwrap();
                let arguments = method
                    .arguments
                    .iter()
                    .map(|x| format!("{} {}", x.type_name, x.name))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(output, "{}", arguments).unwrap();
                write!(output, ")").unwrap();

                if !method.exceptions.is_empty() {
                    write!(output, " throws {}", method.exceptions.join(", ")).unwrap();
                }

                write!(output, " {{").unwrap();
                {
                    let lines: Vec<String> =
                        method.body.lines().map(|x| format!("\t\t{x}")).collect();
                    write!(output, "{}", lines.join("\n")).unwrap();
                }
                write!(output, "\n\t}}\n").unwrap();
            }
        }

        write!(output, "}}").unwrap();

        output
    }
}
