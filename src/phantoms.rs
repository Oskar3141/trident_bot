use rand::{Rng, SeedableRng, rngs::StdRng}; 

// 1:10:34.65-1:19:25.35
const ONE_MINUTE_IN_TICKS: u64 = 1200;
const FIRST_POSSIBLE_SPAWN_TIME_IN_NIGHT: u64 = 693;
const LAST_POSSIBLE_SPAWN_TIME_IN_NIGHT: u64 = 11307;

pub fn get_phantoms_spawn_time() -> (u64, u64) {
    // time for 2 or more phantoms spawn with 2,25 local difficulty
    let mut night: u64 = 4;
    let mut time: u64 = (night * 20 - 10) * ONE_MINUTE_IN_TICKS + FIRST_POSSIBLE_SPAWN_TIME_IN_NIGHT;
    let mut night_end: u64 = (night * 20 - 10) * ONE_MINUTE_IN_TICKS + LAST_POSSIBLE_SPAWN_TIME_IN_NIGHT;
    let local_difficulty: f64 = 2.25;

    let mut rng: StdRng = SeedableRng::from_entropy();

    loop {
        time += rng.gen_range(ONE_MINUTE_IN_TICKS..=(ONE_MINUTE_IN_TICKS * 2));

        if time > night_end {
            night += 1;
            time = (night * 20 - 10) * ONE_MINUTE_IN_TICKS + FIRST_POSSIBLE_SPAWN_TIME_IN_NIGHT;
            night_end = (night * 20 - 10) * ONE_MINUTE_IN_TICKS + LAST_POSSIBLE_SPAWN_TIME_IN_NIGHT;

            continue;
        }

        if rng.gen_range(0.0..3.0) < local_difficulty {
            if rng.gen_range(0.0..1.0) <= (time - 72000) as f64 / time as f64 {
                let spawns: u64 = rng.gen_range(1..=4);
                if spawns >= 2 {
                    return (time, spawns);
                }
            }
        };
    }
}