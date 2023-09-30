use std::fs::File;
use std::io::prelude::*;
use ndarray::{ Array2};
use::hdf5;
pub fn write_mat(phi: &Array2<f64>,name:String,h:f64){
    let mut file = File::create(name+".dat").unwrap();
    for j in  0..phi.shape()[1]{
        for i in  0..phi.shape()[0]{
            writeln!(file, "{:+1.6} {:+1.6} {:+1.6}",(i as f64)*h,(j as f64)*h,phi[[i,j]]).unwrap();
        }
    }
}


pub fn write_stage(data: Vec<&Array2<f64>>,header:Vec<String>,name:String,h:f64){
    let mut file = File::create(name+".dat").unwrap();
    writeln!(file,"{}",header.join(",")).unwrap();
    let shape =data[0].shape();
    let mut row =vec!["".to_string();data.len()+2];
    for j in  0..shape[1]{
        for i in  0..shape[0]{
            row[0] = format!("{:+e}",(i as f64)*h);
            row[1] = format!("{:+e}",(j as f64)*h);
            for  k in 0..data.len(){
                row[k+2]=format!("{:+e}",data[k][[i,j]])
            }
            writeln!(file,"{}",row.join(",")).unwrap();
        }
    }
}


pub fn write_stage_h5(data: Vec<&Array2<f64>>,header:Vec<String>,name:String,h:f64)->hdf5::Result<()>{
    let file = hdf5::File::open_rw("./storag.h5").unwrap();
    let group_name = format!("map/{}",name);
    let group_r = file.create_group(group_name.as_str());
    let group = group_r?;
    for i in 0..data.len(){
        let builder = group.new_dataset::<f32>().shape(data[i].shape());
        let builder = builder.blosc_zstd(9, true);
        let dataset = builder.create(header[i].as_str())?;
        let res = dataset.write((data[i]));
        
    }
    Ok(())
}
pub fn read_stage_h5(data: Vec<&Array2<f64>>,header:Vec<String>)->hdf5::Result<((Vec<Array2<f64>>,f64))>{
    let file = hdf5::File::open_rw("./storag.h5").unwrap();
    let groups_list =  file.group("map")?.groups()?;
    let name = groups_list.last().unwrap().name();
    let time_s:Vec<&str> = name.split('=').collect();
    let time:f64 = time_s[1].parse().unwrap();
    println!("{:?}",name);
    println!("{:?}",time);

    let group_name = format!("{}",name);
    println!("{:?}",group_name);
    let group_r = file.group(group_name.as_str());
    let group = group_r?;
    let mut ret_data: Vec<Array2<f64>> =Vec::new() ;
    for i in 0..data.len(){
        let dataset = group.dataset(header[i].as_str())?;
        ret_data.push(dataset.read()?);
        
    }
    Ok((ret_data,time))
}




