use ::hdf5;
use ndarray::Array2;
use std::fs::File;
use std::io::prelude::*;
pub fn write_mat(phi: &Array2<f64>, name: String, h: f64) {
    let mut file = File::create(name + ".dat").unwrap();
    for j in 0..phi.shape()[1] {
        for i in 0..phi.shape()[0] {
            writeln!(
                file,
                "{:+1.6} {:+1.6} {:+1.6}",
                (i as f64) * h,
                (j as f64) * h,
                phi[[i, j]]
            )
            .unwrap();
        }
    }
}
static STORAGE_PATH: &str = "./storage.h5";

pub fn write_stage(data: Vec<&Array2<f64>>, header: Vec<String>, name: String, h: f64) {
    let mut file = File::create(name + ".dat").unwrap();
    writeln!(file, "{}", header.join(",")).unwrap();
    let shape = data[0].shape();
    let mut row = vec!["".to_string(); data.len() + 2];
    for j in 0..shape[1] {
        for i in 0..shape[0] {
            row[0] = format!("{:+e}", (i as f64) * h);
            row[1] = format!("{:+e}", (j as f64) * h);
            for k in 0..data.len() {
                row[k + 2] = format!("{:+e}", data[k][[i, j]])
            }
            writeln!(file, "{}", row.join(",")).unwrap();
        }
    }
}

pub fn write_stage_h5(
    data: Vec<&Array2<f64>>,
    header: Vec<String>,
    name: String,
    _h: f64,
) -> hdf5::Result<()> {
    let file = hdf5::File::open_rw(STORAGE_PATH).unwrap();
    if !file.link_exists("map") {
        file.create_group("map")?;
    }
    let group_name = format!("map/{}", name);
    if file.link_exists(group_name.as_str()) {
        file.unlink(group_name.as_str())?;
    }
    let group = file.create_group(group_name.as_str())?;
    for i in 0..data.len() {
        //let builder = group.new_dataset::<f32>().shape(data[i].shape());
        //let builder = builder.deflate(8);
        //let dataset = builder.create(header[i].as_str())?;
        //let res = dataset.write((data[i]));
        //println!("res:{:?}", res);
        //    file.new_dataset_builder()
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
            file.unlink(group_name.as_str())?;
            return Err(err);
        }
    }
    Ok(())
}
pub fn read_stage_h5(
    header: Vec<String>,
) -> hdf5::Result<(Vec<Array2<f64>>, f64)> {
    let file = hdf5::File::open_rw(STORAGE_PATH).unwrap();
    let mut stages = Vec::new();
    for group in file.group("map")?.groups()? {
        let name = group.name();
        let Some((_, time_s)) = name.rsplit_once('=') else {
            continue;
        };
        let Ok(time) = time_s.parse::<f64>() else {
            continue;
        };
        if header.iter().all(|name| group.link_exists(name.as_str())) {
            stages.push((time, name));
        }
    }
    stages.sort_by(|a, b| a.0.total_cmp(&b.0));
    let Some((time, group_name)) = stages.last() else {
        return Err("no complete HDF5 stages found".into());
    };
    println!("{:?}", group_name);
    println!("{:?}", time);

    let group = file.group(group_name.as_str())?;
    let mut ret_data: Vec<Array2<f64>> = Vec::new();
    for name in header {
        let dataset = group.dataset(name.as_str())?;
        ret_data.push(dataset.read()?);
    }
    Ok((ret_data, *time))
}
