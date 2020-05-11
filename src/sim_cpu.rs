use crate::bodies;
use crate::output;
use input::SimulationParameters;
use sim_cpu::cowell_perturb::apply_perturbations;
use std::string::ToString;
use strum_macros::Display;

// Gravitational constant 6.674×10−11
const G: f64 = 6.674e-11;

pub struct PerturbationDelta {
    id: u32,
    sim_time: f64,
    acceleration_x_mpss: f64,
    acceleration_y_mpss: f64,
    acceleration_z_mpss: f64,
}

impl Default for PerturbationDelta {
    fn default() -> Self {
        Self {
            id: 0,
            sim_time: 0.0,
            acceleration_x_mpss: 0.0,
            acceleration_y_mpss: 0.0,
            acceleration_z_mpss: 0.0,
        }
    }
}

#[derive(Display)]
pub enum Perturbation {
    #[strum(serialize = "solar_obj")]
    SolarObject(bodies::Solarobj, PerturbationDelta),
}

impl Perturbation {
    fn into_output_form(self) -> output::PerturbationOut {
        output::PerturbationOut {
            id: match &self {
                Perturbation::SolarObject(_, perturb_delta) => perturb_delta.id,
            },
            sim_time: match &self {
                Perturbation::SolarObject(_, perturb_delta) => perturb_delta.sim_time,
            },
            petrub_type: match &self {
                Perturbation::SolarObject(solar_obj, _) => {
                    format!("{}_{}", self.to_string(), solar_obj.to_string())
                }
            },
            acceleration_x_mpss: match &self {
                Perturbation::SolarObject(_, perturb_delta) => perturb_delta.acceleration_x_mpss,
            },
            acceleration_y_mpss: match &self {
                Perturbation::SolarObject(_, perturb_delta) => perturb_delta.acceleration_y_mpss,
            },
            acceleration_z_mpss: match &self {
                Perturbation::SolarObject(_, perturb_delta) => perturb_delta.acceleration_z_mpss,
            },
        }
    }
}

/// Write out all perturbations to output controller.
///
/// ### Arguments
/// * 'perturbations' - Vector containing perturbations to be written out.
/// * 'output_controller' - Controller object used to facilitate the output of perturbation data.
///
fn write_out_all_perturbations(
    perturbations: Vec<Perturbation>,
    output_controller: &mut dyn output::SimulationOutput,
) {
    for perturbation in perturbations {
        output_controller.write_out_perturbation(perturbation.into_output_form());
    }
}

fn write_out_all_object_parameters(
    env: &bodies::Environment,
    sim_objects: &[bodies::SimobjT],
    output_controller: &mut dyn output::SimulationOutput,
) {
    for sim_obj in sim_objects {
        output_controller.write_out_object_parameters(sim_obj.to_output_form(env.sim_time_s));
    }
}

fn write_out_all_solar_objects(
    env: &bodies::Environment,
    output_controller: &mut dyn output::SimulationOutput,
) {
    for solar_object in env.get_solar_objects() {
        output_controller.write_out_solar_object(solar_object.to_output_form(env.sim_time_s));
    }
}

fn l2_norm(x: &ndarray::ArrayView1<f64>) -> f64 {
    x.dot(x).sqrt()
}

fn normalize(x: &ndarray::ArrayView1<f64>, l2_norm_precalc: Option<f64>) -> ndarray::Array1<f64> {
    let norm = match l2_norm_precalc {
        Some(val) => val,
        None => l2_norm(x),
    };
    x.mapv(|e| e / norm)
}

/// Module used to apply perturbation calculations on individual bodies
mod cowell_perturb {
    use crate::bodies;
    use crate::sim_cpu::{Perturbation, PerturbationDelta};
    use bodies::Solarobj;
    use ndarray::{Array1, ArrayView1};
    use sim_cpu::{l2_norm, normalize, G};

    /// Apply all perturbations handled by POSE. This includes:
    /// * 'Solar Body Earth'
    /// * 'Solar Body Moon'
    /// * 'Solar Body Sun'
    /// TODO add more
    ///
    /// ### Parameters
    /// * 'sim_obj' - The object basis for calculation and apply
    /// * 'env' - The Simulation environment
    /// * 'do_return_peturb' - true if vector should be returned, false otherwise
    ///
    /// ### Return
    ///     A vector of perturbation deltas in do_return_peturb is true or none.
    ///
    pub fn apply_perturbations(
        sim_obj: &mut dyn bodies::Simobj,
        env: &bodies::Environment,
        step_time_s: f64,
        do_return_perturb: bool,
    ) -> Option<Vec<Perturbation>> {
        let gravity_perturbations = calc_planet_perturb(sim_obj, env, do_return_perturb);

        let perturbation_vec = vec![gravity_perturbations.0];
        let combined_acceleration = {
            let mut summation = ndarray::Array1::<f64>::zeros(3);
            for element in perturbation_vec {
                summation[0] += element.acceleration_x_mpss;
                summation[1] += element.acceleration_y_mpss;
                summation[2] += element.acceleration_z_mpss;
            }
            summation
        };
        let velocity_delta: Array1<f64> = combined_acceleration * step_time_s;
        let updated_sim_obj_velocity = sim_obj.get_velocity_as_ndarray() + velocity_delta;

        let position_delta = updated_sim_obj_velocity.clone() * step_time_s;
        let updated_sim_obj_coords = sim_obj.get_coords_as_ndarray() + position_delta;

        sim_obj.set_velocity(
            updated_sim_obj_velocity[0],
            updated_sim_obj_velocity[1],
            updated_sim_obj_velocity[2],
        );

        sim_obj.set_coords(
            updated_sim_obj_coords[0],
            updated_sim_obj_coords[1],
            updated_sim_obj_coords[2],
        );

        if !do_return_perturb {
            return None;
        }

        let output_vec = {
            // Upwrap here as this will contain a value at this stage
            let mut result_vec = gravity_perturbations.1.unwrap();
            result_vec
        };

        Some(output_vec)
    }

    fn calc_atmospheric_drag(
        sim_obj: &dyn bodies::Simobj,
        env: &bodies::Environment,
        do_return_perturb: bool,
    ) -> (PerturbationDelta, Perturbation) {
        unimplemented!();
    }

    /// Calculate perturbations due to solar system objects.
    ///
    /// ### Parameters
    /// * 'sim_obj' - The object basis for calculation
    /// * 'env' - The Simulation environment
    /// * 'do_return_peturb' - true if vector should be returned, false otherwise
    ///
    /// ### Return
    ///     A struct of size two containing
    ///         (Total perturbation delta, individual perturbation deltas or none)
    ///
    fn calc_planet_perturb(
        sim_obj: &dyn bodies::Simobj,
        env: &bodies::Environment,
        do_return_perturb: bool,
    ) -> (PerturbationDelta, Option<Vec<Perturbation>>) {
        fn newton_gravitational_field(
            distance_vector: &ArrayView1<f64>,
            planet_idx: usize,
            env: &bodies::Environment,
        ) -> ndarray::Array1<f64> {
            let l2_dist = l2_norm(distance_vector);
            // Calculate unit vector for perturbation
            let unit_vector = normalize(distance_vector, Some(l2_dist));
            // Calculate force using Newton's law of universal gravitation
            let planet_mass_kg = env
                .get_solar_objects()
                .get(planet_idx)
                .expect("Expected in range environment access, invalid index provided.")
                .get_solar_object()
                .get_mass_kg();

            unit_vector * (-G * (planet_mass_kg / l2_dist.powi(2)))
        }

        let mut perturbation_vec = Vec::<Array1<f64>>::with_capacity(env.get_solar_objects().len());
        // Calculate perturbations for each planet object in the environment
        for planet_idx in 0..env.get_solar_objects().len() {
            // Calculate L2 Norm from sim_obj to planet at index planet_index
            let distance_vector = env
                .distance_to(sim_obj, planet_idx)
                .expect("Expected in range environment access, invalid index provided.");
            // Calculate gravity field at position of sim object from planet body
            let mut grav_accel =
                newton_gravitational_field(&distance_vector.view(), planet_idx, env);

            let solar_obj = env
                .get_solar_objects()
                .get(planet_idx)
                .expect("Expected in range environment access, invalid index provided");

            // Special case to handle differential forces on sim object. This is done as
            // simulation objects have positions relative to centric.
            if let Solarobj::Sun { attr: _ } = solar_obj.get_solar_object() {
                if planet_idx != 0 {
                    // Get distance from centric to sun as vector
                    let centric_sun_dist_vector = {
                        let centric_obj_coords = env
                            .get_solar_objects()
                            .get(0)
                            .expect("Expected in range environment access, invalid index provided")
                            .get_coords();
                        let current_obj_coords = solar_obj.get_coords();
                        ndarray::arr1(&[
                            current_obj_coords.xh - centric_obj_coords.xh,
                            current_obj_coords.yh - centric_obj_coords.yh,
                            current_obj_coords.zh - centric_obj_coords.zh,
                        ])
                    };
                    // Calculate gravity field at position of centric
                    let centric_grav = newton_gravitational_field(
                        &centric_sun_dist_vector.view(),
                        planet_idx,
                        env,
                    );

                    // Subtract centric from current
                    grav_accel = grav_accel - centric_grav; // Grav accel on centric
                }
            }

            perturbation_vec.push(grav_accel);
        }

        // Calculate final perturbation
        let sum_perturb = {
            PerturbationDelta {
                id: sim_obj.get_id(),
                sim_time: env.sim_time_s,
                acceleration_x_mpss: perturbation_vec.iter().map(|x| x[0]).sum(),
                acceleration_y_mpss: perturbation_vec.iter().map(|x| x[1]).sum(),
                acceleration_z_mpss: perturbation_vec.iter().map(|x| x[2]).sum(),
            }
        };

        // If per object perturbation calculations are not needed return here
        if !do_return_perturb {
            return (sum_perturb, None);
        }

        let combined_iter = perturbation_vec.iter().zip(env.get_solar_objects());
        let final_perturb_vec = combined_iter
            .map(|(perturb, solar_obj)| {
                Perturbation::SolarObject(
                    solar_obj.get_solar_object().clone(),
                    PerturbationDelta {
                        id: sim_obj.get_id(),
                        sim_time: env.sim_time_s,
                        acceleration_x_mpss: perturb[0],
                        acceleration_y_mpss: perturb[1],
                        acceleration_z_mpss: perturb[2],
                    },
                )
            })
            .collect();

        (sum_perturb, Some(final_perturb_vec))
    }
}

/// Main entry point into the cpu_sim module, gathers all needed data for orbit modeling
/// using Cowell's method.
pub fn simulate(
    mut sim_bodies: Vec<bodies::SimobjT>,
    mut env: bodies::Environment,
    mut output_controller: Box<dyn output::SimulationOutput>,
    sim_params: SimulationParameters,
) {
    loop {
        // Update solar objs
        if env.sim_time_s > env.last_day_update_s + sim_params.sim_solar_step as f64 {
            write_out_all_solar_objects(&env, output_controller.as_mut());
            env.update();
        }

        // Calculate and apply perturbations for every object
        // TODO parallelize this
        for sim_obj in sim_bodies.iter_mut() {
            if let Some(perturb) = apply_perturbations(
                sim_obj.as_mut(),
                &env,
                sim_params.sim_time_step as f64,
                true,
            ) {
                write_out_all_perturbations(perturb, output_controller.as_mut());
            }
        }

        write_out_all_object_parameters(&env, &sim_bodies, output_controller.as_mut());

        // Move forward simulation by step
        env.sim_time_s += sim_params.sim_time_step as f64;
    }
}
