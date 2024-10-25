use std::path::Path;

use clap::Parser;
use serde::{Deserialize, Serialize};

mod cli;
mod compiler;
mod java;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Type {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "i32")]
    I32,
    #[serde(rename = "i64")]
    I64,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "u8")]
    U8,
    #[serde(rename = "i16")]
    I16,
    #[serde(rename = "f32")]
    F32,
    #[serde(rename = "f64")]
    F64,
    #[serde(rename = "self")]
    SelfObject,
    Object(String),
    Array(Box<Type>),
}

impl Type {
    pub fn java_name(&self, object: &Object) -> String {
        match self {
            Type::Bool => "boolean".to_string(),
            Type::I32 => "int".to_string(),
            Type::I64 => "long".to_string(),
            Type::String => "String".to_string(),
            Type::U8 => "byte".to_string(),
            Type::I16 => "short".to_string(),
            Type::F32 => "float".to_string(),
            Type::F64 => "double".to_string(),
            Type::Object(name) => format!("{name}Accessor"),
            Type::Array(array_type) => match array_type.as_ref().clone() {
                Type::Object(name) => format!("{name}Accessor[]"),
                Type::Array(array_type) => format!("{}[][]", array_type.java_name(object)),
                other => format!("{}[]", other.java_name(object)),
            },
            Type::SelfObject => format!("{}Accessor", object.name),
        }
    }

    fn ends_in_object<'a>(&'a self, object: &'a Object) -> Option<&'a str> {
        match self {
            Type::Object(name) => Some(name),
            Type::SelfObject => Some(&object.name),
            Type::Array(array) => array.ends_in_object(object),
            _ => None,
        }
    }

    fn is_nested_array(&self) -> bool {
        if let Type::Array(array) = self {
            if let Type::Array(_) = array.as_ref() {
                return true;
            }
        }
        false
    }

    fn is_primitive(&self) -> bool {
        match self {
            Type::Bool
            | Type::I32
            | Type::I64
            | Type::String
            | Type::U8
            | Type::I16
            | Type::F32
            | Type::F64 => true,
            Type::SelfObject => false,
            Type::Object(_) => false,
            Type::Array(arr) => arr.is_primitive(),
        }
    }

    pub fn generate_accessor(&self, root: bool, object: &Object) -> String {
        if let Some(name) = self.ends_in_object(object) {
            if self.is_nested_array() {
                return format!(
                    "({}){name}Accessor.accessArrayNested(value)",
                    self.java_name(object)
                );
            }

            if let Type::Array(_) = self {
                return format!(
                    "({}){name}Accessor.accessArray((Object[])value)",
                    self.java_name(object)
                );
            }
        }

        let cast = match self {
            Type::Object(name) => format!("{name}Accessor.access(value)"),
            Type::SelfObject => format!("{}Accessor.access(value)", object.name),
            Type::Array(array) => format!("{}[]", array.generate_accessor(false, object)),
            other => other.java_name(object).to_string(),
        };

        if root && self.ends_in_object(object).is_none() {
            format!("({cast})value")
        } else {
            cast
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Field {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
    #[serde(rename = "type")]
    pub field_type: Type,
    #[serde(skip_serializing_if = "is_false", default)]
    pub hierarchy: bool,
}

impl Field {
    pub fn display_name(&self) -> &str {
        self.rename.as_ref().unwrap_or(&self.name)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Variant {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

impl Variant {
    pub fn display_name(&self) -> &str {
        self.rename.as_ref().unwrap_or(&self.name)
    }
}

fn is_false(b: &bool) -> bool {
    !b
}

type Fields = Vec<Field>;
type Variants = Vec<Variant>;

#[derive(Serialize, Deserialize, Debug)]
struct Object {
    pub name: String,
    #[serde(default)]
    pub rename: Option<String>,
    pub package: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub includes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Variants,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Fields,
}

impl Object {
    pub fn display_name(&self) -> &str {
        self.rename.as_ref().unwrap_or(&self.name)
    }
}

#[cfg(debug_assertions)]
fn setup_logging() {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_max_level(tracing::Level::TRACE)
        .init();
}

#[cfg(not(debug_assertions))]
fn setup_logging() {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_max_level(tracing::Level::WARN)
        .init();
}

fn main() -> anyhow::Result<()> {
    let args = cli::Arguments::parse();
    setup_logging();

    match args.command {
        cli::Command::Format { directory } => {
            use colored::*;
            let (changes, _errors) = format(&directory)?;
            if changes == 0 {
                println!("All files checked - {}", "No changes were made".green())
            } else {
                println!(
                    "All files checked - {} {} {}",
                    "Modified".yellow(),
                    changes.to_string().yellow(),
                    "files".yellow()
                );
            }
        }
        cli::Command::Compile { input, output } => match compiler::compile()
            .target(input)
            .output(output)
            .call()
        {
            Ok(files) => {
                for file in files {
                    if let Err(err) = std::fs::write(&file.path, file.content) {
                        tracing::error!("Error writing to file {}: {}", file.path.display(), err);
                    }
                }
            }
            Err(errors) => {
                for error in errors {
                    println!("{}", error);
                }
            }
        },
    }

    Ok(())
}

fn format(dir: &Path) -> anyhow::Result<(i32, i32)> {
    let mut changes = 0;
    let mut errors = 0;

    for path in dir.read_dir()? {
        let path = path?.path();

        if path.is_dir() {
            let (c, e) = format(&path)?;
            changes += c;
            errors += e;
        }

        if path.extension().unwrap_or_default() != "ron" {
            continue;
        }

        let current_content = std::fs::read_to_string(&path)?;
        let current: Result<Object, _> = ron::from_str(&current_content);
        let Ok(current) = current else {
            use colored::*;
            println!(
                "{}: {} {}",
                "ERROR".red(),
                path.display(),
                current.unwrap_err()
            );
            errors += 1;
            continue;
        };

        let expected = ron::ser::to_string_pretty(&current, Default::default())?;

        if expected != current_content {
            use colored::*;
            changes += 1;
            std::fs::write(&path, expected)?;
            println!("{}: {}", "FORMATTED".yellow(), path.display());
        } else {
            use colored::*;
            println!("{}: {}", "OK".green(), path.display());
        }
    }

    Ok((changes, errors))
}
