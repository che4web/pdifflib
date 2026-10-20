use crate::field::Field2D;
use crate::io::{read_stage_h5_for_grid, write_stage, write_stage_h5};
use csv::Writer;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::time::Instant;
const LOG_NAME: &str = "foo.csv";

/// Errors from simulation setup, time-step validation, logging, or checkpointing.
pub type SolveResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub trait System {
    fn next_step(&mut self, _dt: f64, time: f64);
    fn fields(&self) -> Vec<&Field2D>;
    fn fields_mut(&mut self) -> Vec<&mut Field2D>;
    fn get_h(&self) -> f64;
    fn boundary_condition(&mut self);
    fn get_max_time(&self) -> f64;
    fn log_params(&self, wrt: &mut Writer<File>, time: f64);

    fn initial_condition(&mut self);
    fn get_dt(&self) -> f64;
    fn write_mat(&mut self, time: f64) -> hdf5::Result<()> {
        let fields = self.fields();
        let data = fields.iter().map(|field| &field.f).collect();
        let header = fields
            .iter()
            .map(|field| field.name().to_string())
            .collect();
        write_stage_h5(data, header, format!("stage_t={:05}", time), self.get_h())
    }

    /// Restores the latest readable stage with matching field shapes and grid
    /// spacing. Initializes only when storage contains no stages; existing but
    /// unusable checkpoints return an error without changing the fields.
    fn read_mat(&mut self) -> hdf5::Result<f64> {
        let fields = self.fields();
        let header: Vec<_> = fields
            .iter()
            .map(|field| field.name().to_string())
            .collect();
        let shapes: Vec<_> = fields.iter().map(|field| field.f.dim()).collect();
        let Some((data, time)) = read_stage_h5_for_grid(&header, &shapes, self.get_h())? else {
            self.initial_condition();
            return Ok(0.0);
        };

        for (field, value) in self.fields_mut().into_iter().zip(data) {
            field.f = value;
        }
        Ok(time)
    }

    fn write_last(&mut self) -> std::io::Result<()> {
        let fields = self.fields();
        let data = fields.iter().map(|field| &field.f).collect();
        let mut header = vec![String::from("x"), String::from("y")];
        header.extend(fields.iter().map(|field| field.name().to_string()));
        write_stage(data, header, String::from("res/stage_last"), self.get_h())
    }

    /// Runs to `get_max_time`, appending logs and optionally saving checkpoints.
    /// A final checkpoint is always written when `save_map` is true.
    ///
    /// Returns an error on I/O failure, invalid simulation times, or a time step
    /// that is non-finite, non-positive, or too small to advance the clock.
    fn solve(&mut self, save_map: bool) -> SolveResult {
        let max_time = self.get_max_time();
        if !max_time.is_finite() || max_time < 0.0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "maximum time must be finite and non-negative",
            )
            .into());
        }
        if !self.get_h().is_finite() || self.get_h() <= 0.0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "grid spacing must be finite and positive",
            )
            .into());
        }
        let mut time = 0.0;

        let mut wtr: Writer<std::fs::File>;

        {
            let storage_path = "./storage.h5";
            match fs::metadata(storage_path) {
                Ok(_) => {
                    if save_map {
                        let _file = hdf5::File::open_rw(storage_path)?;
                    }
                    time = self.read_mat()?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let _file = hdf5::File::create_excl(storage_path)?;
                    self.initial_condition();
                }
                Err(error) => return Err(error.into()),
            }
            match fs::metadata(LOG_NAME) {
                Ok(_) => {
                    let file = OpenOptions::new().append(true).open(LOG_NAME)?;
                    wtr = Writer::from_writer(file);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    wtr = Writer::from_path(LOG_NAME)?;
                }
                Err(error) => return Err(error.into()),
            }
        }

        let start = Instant::now();
        let mut time_i = 0;
        fs::create_dir_all("res")?;
        if !time.is_finite() || time < 0.0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "restart time must be finite and non-negative",
            )
            .into());
        }
        self.log_params(&mut wtr, time);
        while time < max_time {
            for _i in 0..5000 {
                if time >= max_time {
                    break;
                }
                let requested_dt = self.get_dt();
                if !requested_dt.is_finite() || requested_dt <= 0.0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "time step must be finite and positive",
                    )
                    .into());
                }
                let dt = requested_dt.min(max_time - time);
                let next_time = time + dt;
                if !next_time.is_finite() || next_time <= time {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "time step is too small to advance the simulation clock",
                    )
                    .into());
                }
                self.next_step(dt, time);
                time = next_time;
            }
            time_i += 1;
            self.log_params(&mut wtr, time);
            if time_i % 10 == 0 {
                let duration = start.elapsed();
                println!("{:0.2},{:?}", time / max_time, duration);
                if save_map {
                    self.write_mat(time)?;
                }
            }
        }
        if save_map {
            self.write_mat(time)?;
        }
        wtr.flush()?;
        println!("==============");

        let duration = start.elapsed();
        println!("Time elapsed in expensive_function() is: {:?}", duration);
        Ok(())
    }
}
