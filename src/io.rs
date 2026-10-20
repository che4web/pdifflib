use ::hdf5;
use ndarray::Array2;
use std::fs::File;
use std::io::prelude::*;
pub fn write_mat(phi: &Array2<f64>, name: String, h: f64) -> std::io::Result<()> {
    let mut file = File::create(name + ".dat")?;
    for j in 0..phi.shape()[1] {
        for i in 0..phi.shape()[0] {
            writeln!(
                file,
                "{:+1.6} {:+1.6} {:+1.6}",
                (i as f64) * h,
                (j as f64) * h,
                phi[[i, j]]
            )?;
        }
    }
    file.flush()
}
static STORAGE_PATH: &str = "./storage.h5";

pub fn write_stage(
    data: Vec<&Array2<f64>>,
    header: Vec<String>,
    name: String,
    h: f64,
) -> std::io::Result<()> {
    if data.is_empty()
        || header.len() != data.len() + 2
        || data.iter().any(|field| field.dim() != data[0].dim())
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "stage requires equally shaped fields and headers for x, y, and each field",
        ));
    }
    let mut file = File::create(name + ".dat")?;
    writeln!(file, "{}", header.join(","))?;
    let shape = data[0].shape();
    let mut row = vec!["".to_string(); data.len() + 2];
    for j in 0..shape[1] {
        for i in 0..shape[0] {
            row[0] = format!("{:+e}", (i as f64) * h);
            row[1] = format!("{:+e}", (j as f64) * h);
            for k in 0..data.len() {
                row[k + 2] = format!("{:+e}", data[k][[i, j]])
            }
            writeln!(file, "{}", row.join(","))?;
        }
    }
    file.flush()
}

const PREVIOUS_STAGES: &str = "map/.previous";
const PENDING_STAGE: &str = "map/.pending";

/// Writes a stage with its grid spacing. Replacements retain the previous
/// stage until the new link has been installed and flushed. A readable HDF5
/// file interrupted during replacement can be recovered on the next read/write.
pub fn write_stage_h5(
    data: Vec<&Array2<f64>>,
    header: Vec<String>,
    name: String,
    h: f64,
) -> hdf5::Result<()> {
    if !h.is_finite() || h <= 0.0 {
        return Err("grid spacing must be finite and positive".into());
    }
    if data.is_empty() || header.len() != data.len() {
        return Err("stage requires a name for each field and at least one field".into());
    }
    if name.is_empty() || name.contains('/') || name.contains('\0') || name.starts_with('.') {
        return Err("stage name must be a single non-reserved HDF5 group name".into());
    }
    for (i, field_name) in header.iter().enumerate() {
        if field_name.is_empty()
            || field_name.contains('/')
            || field_name.contains('\0')
            || field_name == "."
            || field_name == ".."
            || header[..i].contains(field_name)
            || data[i].is_empty()
        {
            return Err("fields must be nonempty and have unique single-component names".into());
        }
    }
    let file = hdf5::File::open_rw(STORAGE_PATH)?;
    if !file.link_exists("map") {
        file.create_group("map")?;
    }
    recover_replacements(&file)?;
    let group_name = format!("map/{}", name);
    // Incomplete writes must not be picked up by the restart reader, or replace
    // an existing checkpoint before all datasets have been written successfully.
    if file.link_exists(PENDING_STAGE) {
        file.unlink(PENDING_STAGE)?;
    }
    let group = file.create_group(PENDING_STAGE)?;
    group.new_attr::<f64>().create("h")?.write_scalar(&h)?;
    group
        .new_attr::<u64>()
        .create("field_count")?
        .write_scalar(&(data.len() as u64))?;
    for i in 0..data.len() {
        let rows = data[i].nrows();
        let cols = data[i].ncols();
        let res = group
            .new_dataset_builder()
            .with_data(data[i].view())
            .chunk((rows, cols))
            .shuffle()
            .deflate(4)
            .create(header[i].as_str());
        if let Err(err) = res {
            file.unlink(PENDING_STAGE)?;
            return Err(err);
        }
    }
    file.flush()?;
    let previous_name = format!("{PREVIOUS_STAGES}/{name}");
    if file.link_exists(group_name.as_str()) {
        if !file.link_exists(PREVIOUS_STAGES) {
            file.create_group(PREVIOUS_STAGES)?;
        }
        file.relink(group_name.as_str(), previous_name.as_str())?;
        file.flush()?;
    }
    file.relink(PENDING_STAGE, group_name.as_str())?;
    file.flush()?;
    if file.link_exists(previous_name.as_str()) {
        file.unlink(previous_name.as_str())?;
        file.flush()?;
    }
    Ok(())
}

// A backup may survive either before or after installation of the new stage.
// Restore it when the destination is missing/unreadable; otherwise the new
// stage has already been installed. Do not discard the backup before flushing.
fn recover_replacements(file: &hdf5::File) -> hdf5::Result<()> {
    if !file.link_exists(PREVIOUS_STAGES) {
        return Ok(());
    }
    for name in file.group(PREVIOUS_STAGES)?.member_names()? {
        let destination = format!("map/{name}");
        let backup = format!("{PREVIOUS_STAGES}/{name}");
        let current_is_readable = file
            .group(&destination)
            .and_then(|group| {
                // Installed replacements come from this writer and must carry
                // both metadata attributes. A missing attribute is damage, not
                // a reason to discard a still-usable backup.
                group.attr("h")?;
                group.attr("field_count")?;
                let names = group.member_names()?;
                read_stage_data(&group, &names, None).map(|_| ())
            })
            .is_ok();
        if !current_is_readable {
            // Verify the backup before removing a possibly damaged destination.
            let group = file.group(&backup)?;
            read_stage_data(&group, &group.member_names()?, None)?;
            if file.link_exists(&destination) {
                file.unlink(&destination)?;
            }
            file.relink(&backup, &destination)?;
            file.flush()?;
        } else {
            file.flush()?;
            file.unlink(&backup)?;
        }
    }
    file.flush()
}

type StageData = (Vec<Array2<f64>>, f64);
type GridSpec<'a> = (&'a [(usize, usize)], f64);

/// Reads the latest readable stage, falling back to older stages or replacement
/// backups. This low-level reader also allows legacy stages without `h` metadata;
/// `System` uses a stricter reader that requires matching shapes and spacing.
pub fn read_stage_h5(header: Vec<String>) -> hdf5::Result<(Vec<Array2<f64>>, f64)> {
    read_stages(&header, None)?.ok_or_else(|| "no HDF5 stages found".into())
}

pub(crate) fn read_stage_h5_for_grid(
    header: &[String],
    shapes: &[(usize, usize)],
    h: f64,
) -> hdf5::Result<Option<StageData>> {
    if !h.is_finite() || h <= 0.0 || shapes.len() != header.len() {
        return Err("invalid grid specification for restart".into());
    }
    read_stages(header, Some((shapes, h)))
}

fn read_stages(header: &[String], grid: Option<GridSpec<'_>>) -> hdf5::Result<Option<StageData>> {
    if header.is_empty()
        || header.iter().any(|name| {
            name.is_empty()
                || name.contains('/')
                || name.contains('\0')
                || name == "."
                || name == ".."
        })
    {
        return Err("at least one field name is required".into());
    }
    let file = hdf5::File::open(STORAGE_PATH)?;
    if !file.link_exists("map") {
        return Ok(None);
    }
    let map = file.group("map")?;
    let names = map.member_names()?;
    let mut stages = Vec::new();
    let mut has_stages = names.iter().any(|name| name != ".previous");
    let mut candidates: Vec<_> = names
        .into_iter()
        .filter(|name| !name.starts_with('.'))
        .map(|name| (format!("map/{name}"), name, false))
        .collect();
    if file.link_exists(PREVIOUS_STAGES) {
        for name in file.group(PREVIOUS_STAGES)?.member_names()? {
            has_stages = true;
            candidates.push((format!("{PREVIOUS_STAGES}/{name}"), name, true));
        }
    }
    for (path, name, backup) in candidates {
        let Some((_, time_s)) = name.rsplit_once('=') else {
            continue;
        };
        let Ok(time) = time_s.parse::<f64>() else {
            continue;
        };
        if !time.is_finite() || time < 0.0 {
            continue;
        }
        stages.push((time, path, backup));
    }
    // Prefer the installed stage over its backup at the same timestamp.
    stages.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.2.cmp(&b.2)));
    let mut latest_error = None;
    for (time, path, _) in stages {
        match file
            .group(&path)
            .and_then(|group| read_stage_data(&group, header, grid))
        {
            Ok(data) => return Ok(Some((data, time))),
            Err(error) if latest_error.is_none() => latest_error = Some(format!("{path}: {error}")),
            Err(_) => {}
        }
    }
    if has_stages {
        return Err(format!(
            "no usable HDF5 stage found; {}",
            latest_error
                .as_deref()
                .unwrap_or("no valid stage timestamps")
        )
        .into());
    }
    Ok(None)
}

fn read_stage_data(
    group: &hdf5::Group,
    header: &[String],
    grid: Option<GridSpec<'_>>,
) -> hdf5::Result<Vec<Array2<f64>>> {
    if header.is_empty() {
        return Err("stage has no fields".into());
    }
    let attributes = group.attr_names()?;
    if attributes.iter().any(|name| name == "field_count") {
        let count: u64 = group.attr("field_count")?.read_scalar()?;
        if group.member_names()?.len() as u64 != count {
            return Err("checkpoint field count differs from its metadata".into());
        }
    }
    let h = if attributes.iter().any(|name| name == "h") {
        let h: f64 = group.attr("h")?.read_scalar()?;
        if !h.is_finite() || h <= 0.0 {
            return Err("invalid checkpoint grid spacing".into());
        }
        Some(h)
    } else {
        None
    };
    if let Some((_, expected_h)) = grid {
        if h != Some(expected_h) {
            return Err("checkpoint grid spacing differs or is missing".into());
        }
    }
    let mut data = Vec::with_capacity(header.len());
    for (index, name) in header.iter().enumerate() {
        let values: Array2<f64> = group.dataset(name)?.read()?;
        if values.is_empty() {
            return Err("checkpoint field is empty".into());
        }
        if let Some((shapes, _)) = grid {
            if values.dim() != shapes[index] {
                return Err(format!("checkpoint dimensions differ for field {name}").into());
            }
        }
        data.push(values);
    }
    Ok(data)
}
