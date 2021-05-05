use clap::{self, App, Arg};
use std::{fs, path};
use walkdir::WalkDir;

fn main() {
    let matches = App::new("Image Organizer")
        .author("patric.dexheimer@gmail.com")
        .about("Organize photos/images into folders by date.")
        .arg(
            Arg::with_name("source-folder")
                .help("Sets the input folder")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output-folder")
                .help("Sets the output folder")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("date-mask")
                .help("Set the mask used to create folder")
                .default_value("%Y-%m")
                .index(3),
        )
        .get_matches();

    let folder_input = matches.value_of("source-folder").unwrap();
    let folder_output = matches.value_of("output-folder").unwrap();
    let date_mask = matches.value_of("date-mask").unwrap();

    let files = WalkDir::new(folder_input)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|file| file.path().is_file())
        .collect::<Vec<_>>();

    let total = files.len();
    let mut current = 0;
    for file in files {
        if let Ok(time) = get_file_created_at(file.path()) {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&time, "%Y-%m-%d") {
                let folder_time = date.format(date_mask).to_string();
                let log = copy_file_to_folder(file.path(), folder_output, folder_time);
                println!(
                    "[ {:08} of {:08} ({:.5}%) ] : {}",
                    current,
                    total,
                    (100_f32 / total as f32) * current as f32,
                    log
                );
            }
            current += 1;
        }
    }
}

fn get_file_created_at(file: &path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let file = fs::File::open(file)?;
    let exit_reader = exif::Reader::new();
    let mut bufreader = std::io::BufReader::new(file);
    let meta = exit_reader.read_from_container(&mut bufreader)?;
    let data = meta
        .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
        .ok_or("err")?;

    Ok(data
        .display_value()
        .to_string()
        .split(' ')
        .nth(0)
        .ok_or("err")?
        .trim()
        .to_string())
}

fn copy_file_to_folder(file: &path::Path, folder_output: &str, folder_time: String) -> String {
    let folder = path::Path::new(folder_output).join(folder_time);

    if !folder.is_dir() {
        let _ = fs::create_dir_all(&folder);
    }
    let mut dest = folder.join(file.file_name().unwrap_or_default());

    // this is to not replace repeat images

    // let mut counter = 1;
    // while dest.is_file() {
    //     let name = file.file_stem().unwrap_or_default();
    //     let ext = file.extension().unwrap_or_default();

    //     let mut new_name = name.to_os_string();
    //     new_name.push(format!("_{}.", counter));
    //     new_name.push(ext);

    //     dest = folder.join(new_name);
    //     counter += 1;
    // }

    match fs::copy(file, dest.clone()) {
        Ok(_) => format!(
            "{} -> {}",
            file.to_str().unwrap_or_default(),
            dest.to_str().unwrap_or_default()
        ),

        Err(_) => format!(
            "FAILED: {} -> {}",
            file.to_str().unwrap_or_default(),
            dest.to_str().unwrap_or_default()
        ),
    }
}
