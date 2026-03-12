//! # pigworld
//!
//! An agent-based ecological simulation of pigs on an infinite 2D plane.
//!
//! Each pig has a position (x, y), energy level, and age. On every tick:
//! 1. Pigs move via a Gaussian random walk (cost: `move_cost` energy)
//! 2. Pigs gain `food_per_tick` energy from foraging
//! 3. Nearby pigs (within `interaction_radius`) compete, losing energy
//! 4. Pigs above `reproduction_threshold` energy spawn offspring
//! 5. Pigs with zero energy or exceeding `max_age` die
//!
//! Uses `rayon` for parallel neighbor interaction, `rand`/`rand_distr` for
//! stochastic processes, and `itertools` for histogram aggregation.

use itertools::Itertools;
use miniextendr_api::miniextendr;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

miniextendr_api::miniextendr_init!(pigworld);

/// A single pig agent in the simulation.
///
/// Each pig occupies a position on the infinite 2D plane and tracks its
/// energy (survival resource) and age (ticks survived).
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Pig {
    /// Unique identifier assigned at birth
    id: u64,
    /// X coordinate on the 2D plane
    x: f64,
    /// Y coordinate on the 2D plane
    y: f64,
    /// Current energy level; pig dies when this reaches zero
    energy: f64,
    /// Age in ticks; pig dies when this reaches `max_age`
    age: u32,
}

/// The simulation world containing all pigs and model parameters.
///
/// Wraps the full simulation state including the pig population, the
/// pseudo-random number generator (seeded for reproducibility), and
/// all configurable model parameters. Exposed to R as an opaque
/// `ExternalPtr` object with method dispatch.
#[derive(miniextendr_api::ExternalPtr)]
pub struct World {
    /// All living pigs in the simulation
    pigs: Vec<Pig>,
    /// Current simulation tick (incremented each step)
    tick: u32,
    /// Next unique ID to assign to a newborn pig
    next_id: u64,
    /// Seeded PRNG for reproducible stochastic processes
    rng: rand::rngs::StdRng,
    // ---- Model parameters ----
    /// Energy gained by each pig per tick from foraging
    food_per_tick: f64,
    /// Energy cost per tick for movement
    move_cost: f64,
    /// Energy threshold above which a pig reproduces
    reproduction_threshold: f64,
    /// Maximum age (in ticks) before a pig dies of old age
    max_age: u32,
    /// Pigs within this Euclidean distance compete (lose energy)
    interaction_radius: f64,
}

#[miniextendr]
impl World {
    /// Create a new simulation world with the given parameters.
    ///
    /// Initializes `n_initial` pigs at random positions drawn from a
    /// Normal(0, 5) distribution, each starting with 50 energy.
    ///
    /// @param n_initial Number of pigs to create at time zero.
    /// @param food_per_tick Energy gained by each pig per simulation tick.
    /// @param move_cost Energy spent by each pig per tick for movement.
    /// @param reproduction_threshold Energy level above which a pig reproduces.
    /// @param max_age Maximum age in ticks before a pig dies of old age.
    /// @param interaction_radius Euclidean distance within which pigs compete.
    /// @param seed Integer seed for the random number generator (for reproducibility).
    /// @return A new `World` object (opaque pointer).
    pub fn new(
        n_initial: i32,
        food_per_tick: f64,
        move_cost: f64,
        reproduction_threshold: f64,
        max_age: i32,
        interaction_radius: f64,
        seed: i32,
    ) -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
        let pos_dist = Normal::new(0.0, 5.0).unwrap();

        let mut pigs = Vec::with_capacity(n_initial as usize);
        let mut next_id = 0u64;
        for _ in 0..n_initial {
            pigs.push(Pig {
                id: next_id,
                x: pos_dist.sample(&mut rng),
                y: pos_dist.sample(&mut rng),
                energy: 50.0,
                age: 0,
            });
            next_id += 1;
        }
        World {
            pigs,
            tick: 0,
            next_id,
            rng,
            food_per_tick,
            move_cost,
            reproduction_threshold,
            max_age: max_age as u32,
            interaction_radius,
        }
    }

    /// Advance the simulation by one tick.
    ///
    /// Executes the full tick cycle: movement (Gaussian random walk),
    /// feeding, pairwise interaction (parallel via rayon), reproduction,
    /// and death. Mutates the world in place.
    ///
    /// @return The `World` object (invisibly, for method chaining).
    pub fn step(&mut self) {
        self.tick += 1;
        let move_dist = Normal::new(0.0, 1.0).unwrap();

        // 1. Movement - Normal-distributed random walk
        for pig in &mut self.pigs {
            pig.x += move_dist.sample(&mut self.rng);
            pig.y += move_dist.sample(&mut self.rng);
            pig.energy -= self.move_cost;
            pig.age += 1;
        }

        // 2. Feeding - each pig gains food_per_tick energy
        for pig in &mut self.pigs {
            pig.energy += self.food_per_tick;
        }

        // 3. Interaction - pigs within radius compete (parallel with rayon)
        let positions: Vec<(f64, f64)> = self.pigs.iter().map(|p| (p.x, p.y)).collect();
        let radius_sq = self.interaction_radius * self.interaction_radius;

        let energy_deltas: Vec<f64> = (0..positions.len())
            .into_par_iter()
            .map(|i| {
                let (xi, yi) = positions[i];
                let mut delta = 0.0f64;
                for j in 0..positions.len() {
                    if i == j {
                        continue;
                    }
                    let dx = xi - positions[j].0;
                    let dy = yi - positions[j].1;
                    if dx * dx + dy * dy < radius_sq {
                        delta -= 1.0;
                    }
                }
                delta
            })
            .collect();

        for (pig, &delta) in self.pigs.iter_mut().zip(energy_deltas.iter()) {
            pig.energy += delta;
        }

        // 4. Reproduction
        let mut babies = Vec::new();
        let offspring_dist = Normal::new(0.0, 0.5).unwrap();
        for pig in &self.pigs {
            if pig.energy > self.reproduction_threshold {
                let baby_id = self.next_id;
                self.next_id += 1;
                babies.push(Pig {
                    id: baby_id,
                    x: pig.x + offspring_dist.sample(&mut self.rng),
                    y: pig.y + offspring_dist.sample(&mut self.rng),
                    energy: pig.energy * 0.3,
                    age: 0,
                });
            }
        }
        for pig in &mut self.pigs {
            if pig.energy > self.reproduction_threshold {
                pig.energy *= 0.5;
            }
        }
        self.pigs.extend(babies);

        // 5. Death - remove dead pigs
        self.pigs
            .retain(|p| p.energy > 0.0 && p.age < self.max_age);
    }

    /// Run the simulation for multiple ticks.
    ///
    /// Calls [`step()`] repeatedly. Equivalent to a for-loop calling
    /// `step()` but avoids R-to-Rust call overhead per tick.
    ///
    /// @param steps Number of ticks to advance.
    /// @return The `World` object (invisibly, for method chaining).
    pub fn run(&mut self, steps: i32) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// Get the current simulation tick.
    ///
    /// @return Integer tick count (starts at 0, incremented by each `step()`).
    pub fn get_tick(&self) -> i32 {
        self.tick as i32
    }

    /// Get the number of living pigs.
    ///
    /// @return Integer count of pigs currently alive.
    pub fn population(&self) -> i32 {
        self.pigs.len() as i32
    }

    /// Get all x-coordinates of living pigs.
    ///
    /// @return Numeric vector of x positions (one per pig).
    pub fn x_positions(&self) -> Vec<f64> {
        self.pigs.iter().map(|p| p.x).collect()
    }

    /// Get all y-coordinates of living pigs.
    ///
    /// @return Numeric vector of y positions (one per pig).
    pub fn y_positions(&self) -> Vec<f64> {
        self.pigs.iter().map(|p| p.y).collect()
    }

    /// Get the energy levels of all living pigs.
    ///
    /// @return Numeric vector of energy values (one per pig).
    pub fn energies(&self) -> Vec<f64> {
        self.pigs.iter().map(|p| p.energy).collect()
    }

    /// Get the ages (in ticks) of all living pigs.
    ///
    /// @return Integer vector of ages (one per pig).
    pub fn ages(&self) -> Vec<i32> {
        self.pigs.iter().map(|p| p.age as i32).collect()
    }

    /// Compute the age distribution as a histogram.
    ///
    /// Groups pigs into 10-tick age buckets (0-9, 10-19, ...) and returns
    /// the count per bucket. Uses `itertools::counts()` for aggregation.
    ///
    /// @return Integer vector where element `i` is the number of pigs in
    ///   age bucket `[10*i, 10*i+9]`. Empty vector if no pigs are alive.
    pub fn age_histogram(&self) -> Vec<i32> {
        if self.pigs.is_empty() {
            return Vec::new();
        }
        // Group pigs into age buckets of width 10 using itertools::counts()
        let bucket_counts = self
            .pigs
            .iter()
            .map(|p| (p.age / 10) as usize)
            .counts();
        let max_bucket = bucket_counts.keys().copied().max().unwrap_or(0);
        let mut histogram = vec![0i32; max_bucket + 1];
        for (bucket, count) in &bucket_counts {
            histogram[*bucket] = *count as i32;
        }
        histogram
    }

    /// Get a human-readable summary of the current simulation state.
    ///
    /// @return Character string of the form `"Tick N: M pigs, avg energy X.X"`.
    pub fn summary(&self) -> String {
        let avg_energy = if self.pigs.is_empty() {
            0.0
        } else {
            self.pigs.iter().map(|p| p.energy).sum::<f64>() / self.pigs.len() as f64
        };
        format!(
            "Tick {}: {} pigs, avg energy {:.1}",
            self.tick,
            self.pigs.len(),
            avg_energy
        )
    }
}
