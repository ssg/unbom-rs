use clap::Parser;
use log::{error, info, trace, warn, LevelFilter};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

#[derive(Parser, Debug)]
#[command(version = env!("CARGO_PKG_VERSION"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Params {
    #[arg(short = 'n', long = "nobackup", help = "do not create backup files")]
    nobackup: bool,

    #[arg(required = true, help = "files to process")]
    files: Vec<PathBuf>,
}

const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

fn main() -> Result<(), anyhow::Error> {
    env_logger::builder().filter(None, LevelFilter::Info).init();
    let args = Params::parse_from(wild::args());
    let mut exit_result = Ok(());
    let mut count = 0;

    for filename in &args.files {
        let Ok(mut file) = File::open(filename) else {
            warn!("cannot open file '{}'", filename.display());
            continue;
        };

        if !has_bom(&mut file) {
            trace!("file '{}' does not have a BOM", filename.display());
            continue;
        }

        let result = remove_bom(&file, filename, args.nobackup);
        if result.is_err() {
            exit_result = result;
            continue;
        }
        count += 1;
        trace!("done processing");
    }
    info!("{} file(s) processed", count);

    exit_result
}

fn remove_bom(mut file: &File, filename: &Path, nobackup: bool) -> Result<(), anyhow::Error> {
    info!("processing {}...", filename.display());
    let mut tempfile = create_tempfile(filename, file)?;
    std::io::copy(&mut file, &mut tempfile).inspect_err(|_| error!("cannot write to the temporary file"))?;
    let bak_filename = filename.with_extension("bak");
    std::fs::rename(filename, &bak_filename)
        .inspect_err(|_| error!("cannot create the backup file '{}'", filename.display()))?;
    if std::fs::rename(tempfile.path(), filename).is_err() {
        error!(
            "cannot rename the temporary file to '{}'",
            filename.display()
        );
        std::fs::rename(bak_filename, filename)
            .inspect_err(|_| error!("cannot restore the backup file '{}'", filename.display()))?;
        return Err(anyhow::anyhow!("cannot rename the temporary file"));
    }

    if nobackup {
        std::fs::remove_file(&bak_filename).inspect_err(|_| {
            error!("cannot remove the backup file '{}'", bak_filename.display())
        })?;
    }
    Ok(())
}

fn has_bom<R: Read>(source: &mut R) -> bool {
    let mut buffer = [0; 3];
    source
        .read_exact(&mut buffer)
        .inspect_err(|e| {
            warn!("cannot read the BOM: {}", e);
        })
        .is_ok() && buffer == UTF8_BOM
}

fn create_tempfile(filename: &Path, input_file: &File) -> Result<NamedTempFile, anyhow::Error> {
    let temp_dir = filename.parent().unwrap_or_else(|| Path::new("."));
    let input_permissions = input_file.metadata()?.permissions();
    let mut tempfile = NamedTempFile::new_in(temp_dir)
        .inspect_err(|_| error!("cannot create the temporary file"))?;
    tempfile
        .as_file_mut()
        .set_permissions(input_permissions)
        .inspect_err(|_| error!("cannot set the permissions of the temporary file"))?;
    Ok(tempfile)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_has_bom() {
        let test_cases = vec![
            ("file with BOM and content", vec![0xEF, 0xBB, 0xBF, b'h', b'e', b'l', b'l', b'o'], true),
            ("exactly 3 bytes with BOM", vec![0xEF, 0xBB, 0xBF], true),
            ("file without BOM", vec![b'h', b'e', b'l', b'l', b'o'], false),
            ("empty file", vec![], false),
            ("file too short (1)", vec![0xEF], false),
            ("file too short (2)", vec![0xEF, 0xBB], false),
            ("exactly 3 bytes without BOM", vec![b'a', b'b', b'c'], false),
            ("partial BOM match", vec![0xEF, 0xBB, 0x00, b'h', b'i'], false),
        ];

        for (description, data, expected) in test_cases {
            let mut cursor = Cursor::new(data);
            assert_eq!(
                has_bom(&mut cursor),
                expected,
                "Failed test case: {}",
                description
            );
        }
    }
}