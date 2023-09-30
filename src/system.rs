
use csv::Writer;
use std::time::{Instant};
use std::fs::File;
use std::fs;
use std::fs::OpenOptions;
const LOG_NAME : &str = "foo.csv";

pub trait System{
    fn next_step(&mut self,_dt:f64,time:f64);
    fn write_mat(&mut self, time:f64);
    fn read_mat(&mut self )->f64;
    fn write_last(&mut self);
    fn boundary_condition(&mut self);
    fn get_max_time(&self)->f64;
    fn log_params(&self, wrt:&mut Writer<File>, time:f64);


    fn initial_condition(&mut self);
    fn get_DT(&self) ->f64;
    fn solve(&mut self,save_map:bool){
        let mut time= 0.0;

        let mut wtr: Writer<std::fs::File>; 

        {
            let storage_path = "./storag.h5";
            match fs::metadata(storage_path) {
                Ok(_)=>{
                    time = self.read_mat();
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(LOG_NAME)
                        .unwrap();
                    wtr = Writer::from_writer(file);


                }
                Err(_)=>{
                    let _file = hdf5::File::create("./storag.h5").unwrap();
                    self.initial_condition();
                    wtr = Writer::from_path(LOG_NAME).unwrap();

                }
            }
        }

        let start = Instant::now();
        let mut  time_i=0;
        let r =fs::create_dir("res");
        println!("{:?}",r);
        let DT = self.get_DT();

        while time < self.get_max_time(){
            for _i in 0..5000{
                self.next_step(DT,time);
                time+=DT;
            }
            time_i+=1;
            self.log_params(&mut wtr,time);
            if time_i %10 == 0{
                let duration = start.elapsed();
                println!("{:0.2},{:?}",time/self.get_max_time(),duration) ;
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


