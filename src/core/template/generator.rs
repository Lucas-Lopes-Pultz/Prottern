use super::config::ConfigArg;
use super::engine::parse;
use super::engine::TempEngineArg;
use super::Template;
use crate::cli::input;
use crate::core::template::{TempContent, TempPath, TempPathType};
use crate::utils::string::decode_base64;
use crate::{
    paint, paintln,
    utils::path::{format_path_namespace, pathbuf_to_string, str_to_pathbuf},
};
use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
};

pub fn gen_template(template: Template, directory: &Path) -> Result<(), Error> {

    let template_contents: Result<Vec<TempContent>, Error> = if let Some(args) = template.args {
        template.contents.into_iter().map(|content| {
            let engine_args = get_engine_args(args.clone())?;
            let text_parsed = base64::encode(parse(
                decode_base64(content.text)?,
                engine_args,
            )?);
            Ok(TempContent {
                file_path: content.file_path,
                text: text_parsed
            })
        }).collect()

    } else {
        Ok(template.contents)
    };

    let template_contents = template_contents?;

    paintln!("{gray}", "\n[creating files and folders...]");
    for path in template.paths.into_iter() {
        create_path(path, directory)?;
    }

    if template_contents.len() > 0 {
        paintln!("{gray}", "\n[writing contents...]");
        write_contents(template_contents.clone(), directory)?;
    }

    print!("\n");
    Ok(())
}

fn get_engine_args(args: Vec<ConfigArg>) -> Result<Vec<TempEngineArg>, Error> {
    args.into_iter()
        .map(|arg| {
            let value = input::get(&arg.query)?;
            Ok(TempEngineArg {
                key: arg.key,
                value: if value.is_empty() {
                    if let Some(default) = arg.default {
                        default
                    } else {
                        "".to_string()
                    }
                } else {
                    value
                },
            })
        })
        .collect()
}

fn create_path(path: TempPath, directory: &Path) -> Result<(), Error> {
    let real_path = TempPath {
        path: get_real_path(directory, path.path),
        path_type: path.path_type,
    };

    if real_path.path_type == TempPathType::File {
        fs::write(&real_path.path, "")?;
        paint!("{gray}", "file: ");
    }
    if real_path.path_type == TempPathType::Dir {
        fs::create_dir(&real_path.path)?;
        paint!("{gray}", "dir:  ");
    }

    println!(
        "{}",
        pathbuf_to_string(format_path_namespace(real_path.path))
    );

    Ok(())
}

fn write_contents(contents: Vec<TempContent>, directory: &Path) -> Result<(), Error> {
    for content in contents.into_iter() {
        let file_path = get_real_path(directory, str_to_pathbuf(&content.file_path));
        if file_path.exists() {
            fs::write(&file_path, decode_base64(content.text)?)?;

            print!("{}", pathbuf_to_string(format_path_namespace(file_path)));
            paintln!("...{green}", "ok");
        }
    }

    Ok(())
}

fn get_real_path(directory: &Path, path: PathBuf) -> PathBuf {
    Path::new(directory)
        .join(pathbuf_to_string(path))
        .to_path_buf()
}
