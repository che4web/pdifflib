
use csv::Writer;
use std::time::{Instant};
use std::fs::File;
use std::fs;

pub trait System{
    fn next_step(&mut self,_dt:f64);
    fn write_mat(&mut self, time:f64);
    fn boundary_condition(&mut self);
    fn get_max_time(&self)->f64;
    fn log_params(&self, wrt:&mut Writer<File>, time:f64);
    fn initial_condition(&mut self);
    fn get_DT(&self) ->f64;
    fn solve(&mut self){
        self.initial_condition();
        let mut wtr: Writer<std::fs::File> = Writer::from_path("foo.csv").unwrap();

        let mut time= 0.0;
        let start = Instant::now();
        let mut  time_i=0;
        let r =fs::create_dir("res");
        println!("{:?}",r);
        let DT = self.get_DT();

        while time < self.get_max_time(){
            for _i in 0..1000{
                self.next_step(DT);
                time+=DT;
            }
            time_i+=1;
            self.log_params(&mut wtr,time);
            if time_i %10 == 0{
                let duration = start.elapsed();
                println!("{:0.2},{:?}",time/self.get_max_time(),duration) ;
                self.write_mat(time);
            }
        }
        println!("==============");

         let duration = start.elapsed();
        println!("Time elapsed in expensive_function() is: {:?}", duration);


    }
}


