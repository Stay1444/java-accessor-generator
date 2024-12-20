use std::path::{Path, PathBuf};

use bon::builder;
use ron::de::SpannedError;
use thiserror::Error;

use crate::{java, Object};

mod constructs;

const ALLOWED_EXTENSIONS: &[&str] = &["ron"];

#[derive(Debug)]
pub struct JavaSource {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("An IO error ocurred while reading a source file or directory: {0}")]
    IO(std::io::Error),

    #[error("An error ocurred while deserializing the file: {0}")]
    Deserialization(SpannedError),

    #[error("Object had both fields and variants, the type between enum or class could not be determined.")]
    AmbiguousClassOrEnum,
}

/// Target: the target directory where the original source files are.
/// Will iterate recursively on it and compile everything.
#[builder]
pub fn compile(
    target: impl Into<PathBuf>,
    output: Option<impl Into<PathBuf>>,
) -> Result<Vec<JavaSource>, Vec<(PathBuf, SourceError)>> {
    let target = target.into();
    let output: Option<PathBuf> = output.map(Into::into);

    let read_dir = match target.read_dir() {
        Ok(x) => x,
        Err(err) => {
            return Err(vec![(target, SourceError::IO(err))]);
        }
    };

    let mut failed = vec![];
    let mut completed = vec![];

    // Iterate over every file in the target directory
    for path in read_dir {
        if let Err(err) = path {
            failed.push((PathBuf::new(), SourceError::IO(err)));
            continue;
        };
        let path = path.unwrap().path();

        // If the file is another directory, call the compile function
        // recursively with that directory as the target, and the directory name as the output
        if path.is_dir() {
            let mut output = output.clone().unwrap_or_default();
            let dir_name = path.file_name().expect("dir_name");
            output.push(dir_name);

            match compile().target(&path).output(output).call() {
                Ok(mut x) => completed.append(&mut x),
                Err(mut x) => failed.append(&mut x),
            }
        }

        let extension = match path.extension() {
            None => continue, // If the file does not have an extension just ignore it
            // Try to convert the OsStr into a str if its valid UTF8
            Some(extension) => match extension.to_str() {
                Some(extension) => extension, // Valid UTF8 Extension
                None => continue,             // Invalid UTF8 extension, skip the file
            },
        };

        // Skip files which don't match the allowed extensions
        if !ALLOWED_EXTENSIONS.contains(&extension) {
            tracing::debug!(
                "Skipping file '{}' because it's extension is not allowed",
                path.display()
            );
            continue;
        }

        match compile_file()
            .target(&path)
            .maybe_output(output.as_ref())
            .call()
        {
            Ok(x) => completed.push(x),
            Err(x) => failed.push((path, x)),
        }
    }

    if !failed.is_empty() {
        return Err(failed);
    }
    Ok(completed)
}

#[builder]
fn compile_file(
    target: impl Into<PathBuf>,
    output: Option<impl Into<PathBuf>>,
) -> Result<JavaSource, SourceError> {
    let target = target.into();
    let target_parent = target.parent().expect("File to have a parent");

    let file_name = target
        .file_name()
        .expect("File Name to be present")
        .to_str()
        .expect("File Name to be valid UTF8");

    let source = std::fs::read_to_string(&target).map_err(SourceError::IO)?;
    let output = output.map(Into::into).unwrap_or_default();

    let current: Object = ron::from_str(&source).map_err(SourceError::Deserialization)?;

    // If object has both fields and variants so it's type between enum and class could not be
    // determined.
    if !current.fields.is_empty() && !current.variants.is_empty() {
        return Err(SourceError::AmbiguousClassOrEnum);
    }

    let is_enum = !current.variants.is_empty();

    let output = output.join(format!("{}Accessor.java", current.display_name()));

    let includes: Vec<Object> = current
        .includes
        .iter()
        .map(|x| resolve_include(target_parent, x))
        .collect::<Result<Vec<Object>, SourceError>>()?;

    let source = if is_enum {
        java::Enum::builder()
            .name(format!("{}Accessor", current.display_name()))
            .package(&current.package)
            .comment(format!(
                "Enum autogenerated from {}. DO NOT EDIT.\nOriginal name: {}",
                file_name, current.name
            ))
            .includes(
                includes
                    .iter()
                    .map(|x| format!("{}.{}Accessor", &x.package, x.display_name()))
                    .collect(),
            )
            .variants(
                current
                    .variants
                    .iter()
                    .map(|x| x.display_name().to_owned())
                    .collect(),
            )
            .methods(
                vec![
                    constructs::single_enum_accessor::generate(&current),
                    constructs::array_accessor::generate(&current),
                ]
                .into_iter()
                .collect(),
            )
            .build()
            .source()
    } else {
        java::Class::builder()
            .name(format!("{}Accessor", current.display_name()))
            .package(&current.package)
            .comment(format!(
                "Class autogenerated from {}. DO NOT EDIT\nOriginal name: {}",
                file_name, current.name
            ))
            .includes(
                includes
                    .iter()
                    .map(|x| format!("{}.{}Accessor", &x.package, x.display_name()))
                    .collect(),
            )
            .fields(
                vec![java::Field::builder()
                    .name("self")
                    .type_name("Object")
                    .visibility(java::Visibility::Private)
                    .build()]
                .into_iter()
                .chain(current.fields.iter().map(|x| {
                    java::Field::builder()
                        .name(x.display_name())
                        .type_name(x.field_type.java_name(&current))
                        .visibility(java::Visibility::Public)
                        .maybe_comment(if x.rename.is_some() {
                            Some(format!("Original name: {}", x.name))
                        } else {
                            None
                        })
                        .build()
                }))
                .collect(),
            )
            .methods(
                vec![
                    constructs::class_constructor::generate(&current),
                    constructs::single_class_accessor::generate(&current),
                    constructs::array_accessor::generate(&current),
                    constructs::nested_array_accessor::generate(),
                    constructs::clear_inner_refs::generate(&current, &includes),
                    constructs::clear_inner_refs_array::generate(),
                ]
                .into_iter()
                .chain(constructs::field_setters::generate(
                    &current.fields,
                    &current,
                ))
                .collect(),
            )
            .build()
            .source()
    };

    Ok(JavaSource {
        path: output,
        content: source,
    })
}

/// Turns a path include from a src file into a formatted java include
/// * `working_directory` -  Directory of the current file that is trying to include the other file
/// * `path` - include path as is on the source file
fn resolve_include(
    working_directory: &Path,
    path: impl Into<PathBuf>,
) -> Result<Object, SourceError> {
    let path = path.into();

    let prev_work_dir = std::env::current_dir().map_err(SourceError::IO)?;
    std::env::set_current_dir(working_directory).map_err(SourceError::IO)?;

    let source = std::fs::read_to_string(&path).map_err(SourceError::IO)?;
    let current: Object = ron::from_str(&source).map_err(SourceError::Deserialization)?;

    if let Err(err) = std::env::set_current_dir(prev_work_dir) {
        panic!(
            "Could not change working directory back. Cannot continue.\n{}",
            err
        );
    }

    Ok(current)
}
