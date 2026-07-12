use csv::Writer;
use crate::field::Field2D;
use crate::io::{read_stage_h5, write_stage, write_stage_h5};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::time::Instant;
const LOG_NAME: &str = "foo.csv";

pub trait System {
    fn next_step(&mut self, _dt: f64, time: f64);
    fn fields(&self) -> Vec<&Field2D>;
    fn fields_mut(&mut self) -> Vec<&mut Field2D>;
    fn get_h(&self) -> f64;
    fn boundary_condition(&mut self);
    fn get_max_time(&self) -> f64;
    fn log_params(&self, wrt: &mut Writer<File>, time: f64);

    fn initial_condition(&mut self);
    fn get_DT(&self) -> f64;
    fn write_mat(&mut self, time: f64) {
        let fields = self.fields();
        let data = fields.iter().map(|field| &field.f).collect();
        let header = fields.iter().map(|field| field.name().to_string()).collect();
        let res = write_stage_h5(data, header, format!("stage_t={:05}", time), self.get_h());
        println!("write mat: {:?}", res);
    }

    fn read_mat(&mut self) -> f64 {
        let header = self
            .fields()
            .iter()
            .map(|field| field.name().to_string())
            .collect();
        let ret = read_stage_h5(header);
        println!("read_ from stage");
        println!("{:?}", ret);
        let Ok((data, time)) = ret else {
            self.initial_condition();
            return 0.0;
        };

        let fields = self.fields();
        let same_shapes = fields
            .iter()
            .zip(data.iter())
            .all(|(field, value)| field.f.dim() == value.dim());
        if fields.len() != data.len() || !same_shapes {
            self.initial_condition();
            return 0.0;
        }

        for (field, value) in self.fields_mut().into_iter().zip(data) {
            field.f = value;
        }
        time
    }

    fn write_last(&mut self) {
        let fields = self.fields();
        let data = fields.iter().map(|field| &field.f).collect();
        let mut header = vec![String::from("x"), String::from("y")];
        header.extend(fields.iter().map(|field| field.name().to_string()));
        write_stage(data, header, String::from("res/stage_last"), self.get_h())
    }

    fn solve(&mut self, save_map: bool) {
        let mut time = 0.0;

        let mut wtr: Writer<std::fs::File>;

        {
            let storage_path = "./storage.h5";
            match fs::metadata(storage_path) {
                Ok(_) => {
                    time = self.read_mat();
                }
                Err(_) => {
                    let _file = hdf5::File::create(storage_path).unwrap();
                    self.initial_condition();
                }
            }
            match fs::metadata(LOG_NAME) {
                Ok(_) => {
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(LOG_NAME)
                        .unwrap();
                    wtr = Writer::from_writer(file);
                }
                Err(_) => {
                    wtr = Writer::from_path(LOG_NAME).unwrap();
                }
            }
        }

        let start = Instant::now();
        let mut time_i = 0;
        let r = fs::create_dir("res");
        println!("{:?}", r);
        let DT = self.get_DT();

        while time < self.get_max_time() {
            for _i in 0..500 {
                self.next_step(DT, time);
                time += DT;
            }
            time_i += 1;
            self.log_params(&mut wtr, time);
            if time_i % 10 == 0 {
                let duration = start.elapsed();
                println!("{:0.2},{:?}", time / self.get_max_time(), duration);
                if save_map {
                    self.write_mat(time);
                }
            }
        }
        println!("==============");

        let duration = start.elapsed();
        println!("Time elapsed in expensive_function() is: {:?}", duration);
    }
}
