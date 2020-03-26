//!
//! POSE - Parallel Orbital Simulation Environment
//! TODO - Add more doc

#[macro_use]
mod macros;

extern crate chrono;
extern crate clap;
extern crate serde;
extern crate serde_json;

mod bodies;
mod innout;
mod sim_cpu;

mod cli {

    ///Checks if value passed in to program argument is numeric. Returns a Result
    ///
    ///# Argument
    ///* 'strng' - The value passed by the user
    fn numeric_validator(strng: String) -> Result<(), String> {
        if strng.parse::<f32>().is_ok() {
            Ok(())
        } else {
            Err(String::from("Input is non-numeric"))
        }
    }

    /// Defines the argument structure for the pose simulation program
    /// Returns the result of user arguments passed over the cli
    pub fn check_cli() -> clap::ArgMatches<'static> {
        // Defines the input arguments from the cli
        let matches = clap::App::new("Parallel Orbital Simulation Environment (POSE)")
            .version("DEV0.1")
            .about("Simulation aimed to model the orbital environment around Earth for bodies at all magnitudes.")
            .args(&[
                clap::Arg::with_name("INPUT")
                    .help("json file containing information on bodies at initialization.")
                    .required(true)
                    .index(1),
                clap::Arg::with_name("out")
                    .help("Directory to place output files.")
                    .short("o")
                    .long("out")
                    .value_name("DIR_NAME")
                    .takes_value(true),
                clap::Arg::with_name("sim_time_step")
                    .help("Simulation time step interval in seconds")
                    .short("s")
                    .long("step")
                    .value_name("STEP_INTERVAL")
                    .takes_value(true)
                    .validator(numeric_validator)
            ])
            .get_matches();

        return matches;
    }
}

fn main() {
    let matches = cli::check_cli();
    let sim_params = innout::gather_program_arguments(matches);

    let (sim_bodies, day) = innout::parse_inpt(sim_params.input_bodies_json.as_str());
    let env = bodies::Environment::new(day);

    sim_cpu::simulate(sim_bodies, env);
}
