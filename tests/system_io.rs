use ndarray::Array2;
use pdifflib::field::Field2D;
use pdifflib::io::{read_stage_h5, write_stage, write_stage_h5};
use pdifflib::system::System;
use std::fs::{self, File};

// The public API uses relative paths. Run each scenario in its own process and
// temporary working directory, without changing the test runner's global cwd.
fn isolated(name: &str) -> bool {
    if std::env::var("PDIFFLIB_TEST_CHILD").as_deref() == Ok(name) {
        return true;
    }
    let path = std::env::temp_dir().join(format!(
        "pdifflib-{}-{}-{}",
        std::process::id(),
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("PDIFFLIB_TEST_CHILD", name)
        .current_dir(&path)
        .output()
        .unwrap();
    fs::remove_dir_all(&path).unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    false
}

struct Simulation {
    u: Field2D,
    dt: f64,
    max_time: f64,
    steps: usize,
    fail_write: bool,
    h: f64,
    initialized: bool,
}

impl Simulation {
    fn new(dt: f64, max_time: f64) -> Self {
        Self {
            u: Field2D::new("u", 5, 7),
            dt,
            max_time,
            steps: 0,
            fail_write: false,
            h: 1.0,
            initialized: false,
        }
    }
}

impl System for Simulation {
    fn next_step(&mut self, dt: f64, _: f64) {
        self.steps += 1;
        self.u.f[[1, 1]] += dt;
    }
    fn fields(&self) -> Vec<&Field2D> {
        vec![&self.u]
    }
    fn fields_mut(&mut self) -> Vec<&mut Field2D> {
        vec![&mut self.u]
    }
    fn get_h(&self) -> f64 {
        self.h
    }
    fn boundary_condition(&mut self) {}
    fn get_max_time(&self) -> f64 {
        self.max_time
    }
    fn log_params(&self, writer: &mut csv::Writer<File>, time: f64) {
        writer.write_record([time.to_string()]).unwrap();
    }
    fn initial_condition(&mut self) {
        self.initialized = true;
        self.u.f.fill(0.0);
    }
    fn get_dt(&self) -> f64 {
        self.dt
    }
    fn write_mat(&mut self, time: f64) -> hdf5::Result<()> {
        if self.fail_write {
            return Err("simulated checkpoint failure".into());
        }
        write_stage_h5(
            vec![&self.u.f],
            vec!["u".into()],
            format!("stage_t={time}"),
            self.get_h(),
        )
    }
}

#[test]
fn final_checkpoint_and_restart() {
    if !isolated("final_checkpoint_and_restart") {
        return;
    }
    let mut simulation = Simulation::new(0.75, 2.0);
    simulation.solve(true).unwrap();
    assert_eq!(simulation.steps, 3);
    let (fields, time) = read_stage_h5(vec!["u".into()]).unwrap();
    assert_eq!(time, 2.0);
    assert_eq!(fields[0][[1, 1]], 2.0);

    let mut resumed = Simulation::new(0.75, 3.0);
    resumed.solve(true).unwrap();
    assert_eq!(resumed.steps, 2);
    assert!(!resumed.initialized);
    assert_eq!(resumed.u.f[[1, 1]], 3.0);
    assert_eq!(read_stage_h5(vec!["u".into()]).unwrap().1, 3.0);
    assert_eq!(fs::read_to_string("foo.csv").unwrap().lines().count(), 4);
}

#[test]
fn invalid_steps_return_errors_without_advancing() {
    if !isolated("invalid_steps_return_errors_without_advancing") {
        return;
    }
    for dt in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut simulation = Simulation::new(dt, 1.0);
        let error = simulation.solve(false).unwrap_err();
        assert!(error.to_string().contains("time step"));
        assert_eq!(simulation.steps, 0);
    }
    for max_time in [-1.0, f64::NAN, f64::INFINITY] {
        let mut simulation = Simulation::new(1.0, max_time);
        assert!(simulation.solve(false).is_err());
        assert_eq!(simulation.steps, 0);
    }

    let values = Array2::zeros((5, 7));
    write_stage_h5(vec![&values], vec!["u".into()], "stage_t=1".into(), 1.0).unwrap();
    let mut simulation = Simulation::new(f64::MIN_POSITIVE, 2.0);
    let error = simulation.solve(false).unwrap_err();
    assert!(error.to_string().contains("too small"));
    assert_eq!(simulation.steps, 0);
}

#[test]
fn checkpoint_errors_reach_the_caller() {
    if !isolated("checkpoint_errors_reach_the_caller") {
        return;
    }
    for max_time in [1.0, 50_001.0] {
        let mut simulation = Simulation::new(1.0, max_time);
        simulation.fail_write = true;
        let error = simulation.solve(true).unwrap_err();
        assert!(error.to_string().contains("simulated checkpoint failure"));
        assert_eq!(simulation.steps, max_time.min(50_000.0) as usize);
    }
}

#[test]
fn io_errors_do_not_panic_or_destroy_old_checkpoints() {
    if !isolated("io_errors_do_not_panic_or_destroy_old_checkpoints") {
        return;
    }
    let values = Array2::from_elem((5, 7), 42.0);
    assert!(read_stage_h5(vec!["u".into()]).is_err());
    assert!(write_stage_h5(vec![&values], vec!["u".into()], "stage_t=1".into(), 1.0).is_err());
    fs::write("storage.h5", "not an HDF5 file").unwrap();
    let mut simulation = Simulation::new(1.0, 2.0);
    assert!(simulation.solve(true).is_err());
    assert_eq!(simulation.steps, 0);
    fs::remove_file("storage.h5").unwrap();

    hdf5::File::create("storage.h5").unwrap();
    write_stage_h5(vec![&values], vec!["u".into()], "stage_t=1".into(), 1.0).unwrap();
    assert!(write_stage_h5(
        vec![&values],
        vec!["bad\0name".into()],
        "stage_t=1".into(),
        1.0
    )
    .is_err());
    let (restored, time) = read_stage_h5(vec!["u".into()]).unwrap();
    assert_eq!(time, 1.0);
    assert_eq!(restored[0], values);
    assert!(write_stage_h5(vec![&values], vec![], "stage_t=2".into(), 1.0).is_err());
    assert!(write_stage(vec![], vec![], "empty".into(), 1.0).is_err());
    assert!(pdifflib::io::write_mat(&values, "missing/directory/file".into(), 1.0).is_err());
}

#[test]
fn restart_ignores_incomplete_and_nonfinite_stages() {
    if !isolated("restart_ignores_incomplete_and_nonfinite_stages") {
        return;
    }
    let values = Array2::from_elem((5, 7), 42.0);
    hdf5::File::create("storage.h5").unwrap();
    write_stage_h5(vec![&values], vec!["u".into()], "stage_t=2".into(), 1.0).unwrap();
    for name in ["stage_t=NaN", "stage_t=inf", "stage_t=-1", "stage_t=3"] {
        let file = hdf5::File::open_rw("storage.h5").unwrap();
        let group = file.create_group(&format!("map/{name}")).unwrap();
        if name != "stage_t=3" {
            group
                .new_dataset_builder()
                .with_data(&values)
                .create("u")
                .unwrap();
        }
    }
    assert_eq!(read_stage_h5(vec!["u".into()]).unwrap().1, 2.0);

    let mut simulation = Simulation::new(1.0, 3.0);
    simulation.u = Field2D::new("u", 3, 3);
    assert!(simulation.read_mat().is_err());
    assert!(!simulation.initialized);
    assert!(simulation.u.f.iter().all(|&value| value == 0.0));
}

#[test]
fn log_open_errors_are_returned() {
    if !isolated("log_open_errors_are_returned") {
        return;
    }
    fs::create_dir("foo.csv").unwrap();
    let mut simulation = Simulation::new(1.0, 2.0);
    assert!(simulation.solve(false).is_err());
    assert_eq!(simulation.steps, 0);
}

fn scalar_stage(name: &str) {
    let file = hdf5::File::open_rw("storage.h5").unwrap();
    let group = file.create_group(&format!("map/{name}")).unwrap();
    group
        .new_attr::<f64>()
        .create("h")
        .unwrap()
        .write_scalar(&1.0)
        .unwrap();
    group
        .new_dataset::<f64>()
        .shape(())
        .create("u")
        .unwrap()
        .write_scalar(&7.0)
        .unwrap();
}

#[test]
fn restart_falls_back_from_unreadable_or_incompatible_newer_stages() {
    if !isolated("restart_falls_back_from_unreadable_or_incompatible_newer_stages") {
        return;
    }
    hdf5::File::create("storage.h5").unwrap();
    let values = Array2::from_elem((5, 7), 42.0);
    write_stage_h5(vec![&values], vec!["u".into()], "stage_t=1".into(), 1.0).unwrap();
    write_stage_h5(vec![&values], vec!["u".into()], "stage_t=2".into(), 2.0).unwrap();
    let different_shape = Array2::zeros((3, 3));
    write_stage_h5(
        vec![&different_shape],
        vec!["u".into()],
        "stage_t=3".into(),
        1.0,
    )
    .unwrap();
    scalar_stage("stage_t=4");

    // The raw reader checks readability; System also checks shape and spacing.
    assert_eq!(read_stage_h5(vec!["u".into()]).unwrap().1, 3.0);
    let mut simulation = Simulation::new(1.0, 2.0);
    assert_eq!(simulation.read_mat().unwrap(), 1.0);
    assert_eq!(simulation.u.f, values);
    assert!(!simulation.initialized);
}

#[test]
fn unusable_checkpoints_fail_without_initializing_or_overwriting() {
    if !isolated("unusable_checkpoints_fail_without_initializing_or_overwriting") {
        return;
    }
    hdf5::File::create("storage.h5").unwrap();
    scalar_stage("stage_t=1");
    let mut simulation = Simulation::new(1.0, 2.0);
    simulation.u.f.fill(123.0);
    assert!(simulation.solve(true).is_err());
    assert!(!simulation.initialized);
    assert_eq!(simulation.steps, 0);
    assert!(simulation.u.f.iter().all(|&value| value == 123.0));
    let file = hdf5::File::open("storage.h5").unwrap();
    assert_eq!(file.dataset("map/stage_t=1/u").unwrap().ndim(), 0);
    assert!(!file.link_exists("map/stage_t=2"));
}

#[test]
fn grid_spacing_is_persisted_and_required_for_automatic_restart() {
    if !isolated("grid_spacing_is_persisted_and_required_for_automatic_restart") {
        return;
    }
    hdf5::File::create("storage.h5").unwrap();
    let values = Array2::from_elem((5, 7), 42.0);
    write_stage_h5(vec![&values], vec!["u".into()], "stage_t=1".into(), 0.25).unwrap();
    {
        let file = hdf5::File::open("storage.h5").unwrap();
        assert_eq!(
            file.group("map/stage_t=1")
                .unwrap()
                .attr("h")
                .unwrap()
                .read_scalar::<f64>()
                .unwrap(),
            0.25
        );
    }
    let mut simulation = Simulation::new(1.0, 2.0);
    let error = simulation.solve(true).unwrap_err();
    assert!(error.to_string().contains("grid spacing"));
    assert!(!simulation.initialized);
    assert_eq!(simulation.steps, 0);
    simulation.h = 0.25;
    assert_eq!(simulation.read_mat().unwrap(), 1.0);

    for h in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(write_stage_h5(vec![&values], vec!["u".into()], "stage_t=2".into(), h).is_err());
        simulation.h = h;
        assert!(simulation.solve(true).is_err());
    }
}

#[test]
fn legacy_spacing_must_be_supplied_explicitly_before_restart() {
    if !isolated("legacy_spacing_must_be_supplied_explicitly_before_restart") {
        return;
    }
    let values = Array2::from_elem((5, 7), 42.0);
    {
        let file = hdf5::File::create("storage.h5").unwrap();
        let group = file.create_group("map/stage_t=1").unwrap();
        group
            .new_dataset_builder()
            .with_data(&values)
            .create("u")
            .unwrap();
    }
    assert_eq!(read_stage_h5(vec!["u".into()]).unwrap().0[0], values);
    let mut simulation = Simulation::new(1.0, 2.0);
    assert!(simulation
        .read_mat()
        .unwrap_err()
        .to_string()
        .contains("grid spacing"));
    assert!(!simulation.initialized);
    // Explicit conversion through the writer records the known original spacing.
    write_stage_h5(vec![&values], vec!["u".into()], "stage_t=1".into(), 1.0).unwrap();
    assert_eq!(simulation.read_mat().unwrap(), 1.0);
}

#[test]
fn interrupted_replacement_remains_readable_and_is_recovered_on_write() {
    if !isolated("interrupted_replacement_remains_readable_and_is_recovered_on_write") {
        return;
    }
    let previous = Array2::from_elem((5, 7), 42.0);
    let pending = Array2::from_elem((5, 7), 99.0);
    let final_values = Array2::from_elem((5, 7), 100.0);
    // Simulate interruption before installation, after installation, and with
    // an unreadable dataset or a missing field. Reads do not require write access.
    for (installed, damaged, field_count) in [
        (false, false, 1),
        (true, false, 1),
        (true, true, 1),
        (true, false, 2),
    ] {
        hdf5::File::create("storage.h5").unwrap();
        write_stage_h5(vec![&previous], vec!["u".into()], "stage_t=1".into(), 1.0).unwrap();
        {
            let file = hdf5::File::open_rw("storage.h5").unwrap();
            file.create_group("map/.previous").unwrap();
            file.relink("map/stage_t=1", "map/.previous/stage_t=1")
                .unwrap();
            let group = file.create_group("map/.pending").unwrap();
            group
                .new_attr::<u64>()
                .create("field_count")
                .unwrap()
                .write_scalar(&field_count)
                .unwrap();
            group
                .new_attr::<f64>()
                .create("h")
                .unwrap()
                .write_scalar(&1.0)
                .unwrap();
            if damaged {
                group
                    .new_dataset::<f64>()
                    .shape(())
                    .create("u")
                    .unwrap()
                    .write_scalar(&7.0)
                    .unwrap();
            } else {
                group
                    .new_dataset_builder()
                    .with_data(&pending)
                    .create("u")
                    .unwrap();
            }
            if installed {
                file.relink("map/.pending", "map/stage_t=1").unwrap();
            }
            file.flush().unwrap();
        }
        let expected = if installed && !damaged && field_count == 1 {
            &pending
        } else {
            &previous
        };
        assert_eq!(&read_stage_h5(vec!["u".into()]).unwrap().0[0], expected);
        let mut simulation = Simulation::new(1.0, 2.0);
        assert_eq!(simulation.read_mat().unwrap(), 1.0);
        assert_eq!(&simulation.u.f, expected);
        assert!(!simulation.initialized);

        write_stage_h5(
            vec![&final_values],
            vec!["u".into()],
            "stage_t=1".into(),
            1.0,
        )
        .unwrap();
        assert_eq!(read_stage_h5(vec!["u".into()]).unwrap().0[0], final_values);
        let file = hdf5::File::open("storage.h5").unwrap();
        assert!(!file.link_exists("map/.previous/stage_t=1"));
        assert!(!file.link_exists("map/.pending"));
    }
}
