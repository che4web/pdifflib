use std::fs::File;
use std::io::prelude::*;
use ndarray::{ Array2};
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
            row[0] = format!("{:+1.6}",(i as f64)*h);
            row[1] = format!("{:+1.6}",(j as f64)*h);
            for  k in 0..data.len(){
                row[k+2]=format!("{:+1.6}",data[k][[i,j]])
            }
            writeln!(file,"{}",row.join(",")).unwrap();
        }
    }
}


