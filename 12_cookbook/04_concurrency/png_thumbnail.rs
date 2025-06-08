use glob::{MatchOptions, glob_with};
use image::imageops::FilterType;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{fs::create_dir_all, path::Path};

fn make_thumbnail<PA, PB>(original: PA, thumb_dir: PB, longest_edge: u32) -> Result<(), ()>
where
    PA: AsRef<Path>,
    PB: AsRef<Path>,
{
    let img = image::open(original.as_ref()).map_err(|_| ())?;
    let original = original.as_ref().file_name().expect("Not an image");
    let output = thumb_dir.as_ref().join(original);
    img.resize(longest_edge, longest_edge, FilterType::Nearest)
        .save(output)
        .map_err(|_| ())
}

fn main() {
    let options = MatchOptions::default();
    let files = glob_with("**/*.png", options)
        .expect("Cannot find pngs")
        .filter_map(|x| x.ok())
        .collect::<Vec<_>>();

    if files.len() == 0 {
        println!("There is no pngs");
        return;
    }

    let thumb_dir = "/tmp/thumbnails";
    create_dir_all(thumb_dir).expect("Fail to create dir");
    println!("Saving {} thumbnails into '{}'...", files.len(), thumb_dir);

    let failures: Vec<_> = files
        .par_iter()
        .map(|path| make_thumbnail(path, thumb_dir, 200).map_err(|_| format!("Error: {:?}", path)))
        .filter_map(|x| x.err())
        .collect();
    failures.iter().for_each(|e| println!("{}", e));
    println!(
        "{} thumbnails saved successfully",
        files.len() - failures.len()
    );
}
