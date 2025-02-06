use std::{
    fs::{remove_file, rename, File},
    io::{copy, Read},
    path::{Path, PathBuf},
};

use clap::Parser;
use log::{error, info, trace, warn, LevelFilter};
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
    let mut last_error: Option<anyhow::Error> = None;
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

        if let Err(e) = remove_bom(&file, filename, args.nobackup) {
            last_error = Some(e);
            continue;
        }        
        count += 1;
        trace!("done processing");
    }
    info!("{} file(s) processed", count);

    if let Some(e) = last_error { Err(e) } else { Ok(()) }
}

fn remove_bom(mut file: &File, filename: &Path, nobackup: bool) -> Result<(), anyhow::Error> {
    info!("processing {}...", filename.display());
    let mut tempfile = create_tempfile(filename, file)?;
    copy(&mut file, &mut tempfile)
        .inspect_err(|_| error!("cannot write to the temporary file"))?;
    let bak_filename = filename.with_extension("bak");
    rename(filename, &bak_filename)
        .inspect_err(|_| error!("cannot create the backup file '{}'", filename.display()))?;
    if rename(tempfile.path(), filename).is_err() {
        error!(
            "cannot rename the temporary file to '{}'",
            filename.display()
        );
        rename(bak_filename, filename).inspect_err(|_| {
            error!("cannot restore the backup file '{}'", filename.display())
        })?;
        return Err(anyhow::anyhow!("cannot rename the temporary file"));
    }

    if nobackup {
        remove_file(&bak_filename).inspect_err(|_| {
            error!("cannot remove the backup file '{}'", bak_filename.display())
        })?;
    }
    Ok(())
}

fn has_bom(file: &mut dyn Read) -> bool {
    let mut buffer = [0; 3];
    if file.read_exact(&mut buffer).inspect_err(|e| {
        warn!("cannot read the BOM: {}", e);
    }).is_err() {
        return false;
    }
    buffer == UTF8_BOM
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
