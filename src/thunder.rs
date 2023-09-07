use rand::{Rng, SeedableRng, rngs::StdRng}; 

const ONE_MINUTE_IN_TICKS: u64 = 1200;

const MIN_TIME_BETWEEN_CYCLES: u64 = ONE_MINUTE_IN_TICKS * 10;
const MAX_TIME_BETWEEN_CYCLES: u64 = ONE_MINUTE_IN_TICKS * 150;

const MIN_RAIN_CYCLE_DURATION: u64 = ONE_MINUTE_IN_TICKS * 10;
const MAX_RAIN_CYCLE_DURATION: u64 = ONE_MINUTE_IN_TICKS * 20;
const MIN_THUNDER_CYCLE_DURATION: u64 = ONE_MINUTE_IN_TICKS * 3;
const MAX_THUNDER_CYCLE_DURATION: u64 = ONE_MINUTE_IN_TICKS * 13;

pub fn get_thunder_duration(
    rain_cycle_start: u64,
    rain_cycle_duration: u64,
    thunder_cycle_start: u64,
    thunder_cycle_duration: u64
) -> u64 {
    let rain_cycle_end: u64 = rain_cycle_start + rain_cycle_duration;
    let thunder_cycle_end: u64 = thunder_cycle_start + thunder_cycle_duration;

    // if rain_cycle_start > thunder_cycle_end || thunder_cycle_start > rain_cycle_end {
    //     return 0;
    // }

    // if rain_cycle_start >= thunder_cycle_start && rain_cycle_end <= thunder_cycle_end {
    //     return rain_cycle_end - rain_cycle_start;
    // } else if thunder_cycle_start >= rain_cycle_start && thunder_cycle_end <= rain_cycle_end {
    //     return thunder_cycle_end - thunder_cycle_start;
    // } else if thunder_cycle_start <= rain_cycle_end && thunder_cycle_end >= rain_cycle_end {
    //     return rain_cycle_end - thunder_cycle_start;
    // } else if rain_cycle_start <= thunder_cycle_end && rain_cycle_end >= thunder_cycle_end {
    //     return thunder_cycle_end - rain_cycle_start;
    // }

    if rain_cycle_start < thunder_cycle_start && thunder_cycle_end < rain_cycle_end {
        return thunder_cycle_duration;
    } else if thunder_cycle_start < rain_cycle_start && rain_cycle_end < thunder_cycle_end {
        return rain_cycle_duration;
    } else if thunder_cycle_start > rain_cycle_start && thunder_cycle_start < rain_cycle_end {
        return rain_cycle_end - thunder_cycle_start;
    } else if rain_cycle_start > thunder_cycle_start && rain_cycle_start < thunder_cycle_end {
        return thunder_cycle_end - rain_cycle_start;
    } else if thunder_cycle_end > rain_cycle_start && thunder_cycle_end < rain_cycle_end {
        return thunder_cycle_end - rain_cycle_start;
    } else if rain_cycle_end > thunder_cycle_start && rain_cycle_end < thunder_cycle_end {
        return rain_cycle_end - thunder_cycle_start;
    }

    // println!("{}, {}, {}, {}, {}, {}", 
    //     rain_cycle_start as f64 / ONE_MINUTE_IN_TICKS as f64, 
    //     rain_cycle_duration as f64 / ONE_MINUTE_IN_TICKS as f64, 
    //     thunder_cycle_start as f64 / ONE_MINUTE_IN_TICKS as f64, 
    //     thunder_cycle_duration as f64 / ONE_MINUTE_IN_TICKS as f64, 
    //     rain_cycle_end as f64 / ONE_MINUTE_IN_TICKS as f64, 
    //     thunder_cycle_end as f64 / ONE_MINUTE_IN_TICKS as f64
    // );

    0
}

pub fn get_thunder_start_time(
    rain_cycle_start: u64,
    thunder_cycle_start: u64
) -> u64 {
    if rain_cycle_start > thunder_cycle_start {
        rain_cycle_start
    } else {
        thunder_cycle_start
    }
}

pub fn get_thunder_odds(time: u64) -> f64 {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let loops: u64 = 1_000_000;
    let mut succes: u64 = 0;

    for _ in 1..=loops {
        let mut rain_cycle_start: u64 = rng.gen_range((MIN_TIME_BETWEEN_CYCLES + 1)..MAX_TIME_BETWEEN_CYCLES);
        let mut rain_cycle_duration: u64 = rng.gen_range(MIN_RAIN_CYCLE_DURATION..MAX_RAIN_CYCLE_DURATION);
        let mut thunder_cycle_start: u64 = rng.gen_range((MIN_TIME_BETWEEN_CYCLES + 1)..MAX_TIME_BETWEEN_CYCLES);
        let mut thunder_cycle_duration: u64 = rng.gen_range(MIN_THUNDER_CYCLE_DURATION..MAX_THUNDER_CYCLE_DURATION);
        let mut start_time: u64;

        'thunder_loop: loop {
            let duration: u64 = get_thunder_duration(
                rain_cycle_start,
                rain_cycle_duration,
                thunder_cycle_start,
                thunder_cycle_duration
            );

            start_time = get_thunder_start_time(rain_cycle_start, thunder_cycle_start);
            if duration > 0 && start_time < time {
                succes += 1;
                break 'thunder_loop;
            } else if thunder_cycle_start >= time || rain_cycle_start >= time {
                break 'thunder_loop;
            }

            if rain_cycle_start <= thunder_cycle_start {
                let rain_cycle_end: u64 = rain_cycle_start + rain_cycle_duration;
                
                rain_cycle_start = rain_cycle_end + rng.gen_range((MIN_TIME_BETWEEN_CYCLES + 1)..MAX_TIME_BETWEEN_CYCLES);
                rain_cycle_duration = rng.gen_range(MIN_RAIN_CYCLE_DURATION..MAX_RAIN_CYCLE_DURATION);
            } else {
                let thunder_cycle_end: u64 = thunder_cycle_start + thunder_cycle_duration;
                
                thunder_cycle_start = thunder_cycle_end + rng.gen_range((MIN_TIME_BETWEEN_CYCLES + 1)..MAX_TIME_BETWEEN_CYCLES);
                thunder_cycle_duration = rng.gen_range(MIN_THUNDER_CYCLE_DURATION..MAX_THUNDER_CYCLE_DURATION);
            }
        }

        // while  == 0 {
            // if rain_cycle_start <= thunder_cycle_start {
            //     let rain_cycle_end: u64 = rain_cycle_start + rain_cycle_duration;
                
            //     rain_cycle_start = rng.gen_range((rain_cycle_end + MIN_TIME_BETWEEN_CYCLES + 1)..=(rain_cycle_end + MAX_TIME_BETWEEN_CYCLES));
            //     rain_cycle_duration = rng.gen_range(MIN_RAIN_CYCLE_DURATION..=MAX_RAIN_CYCLE_DURATION);
            // } else {
            //     let thunder_cycle_end: u64 = thunder_cycle_start + thunder_cycle_duration;
                
            //     thunder_cycle_start = rng.gen_range((thunder_cycle_end + MIN_TIME_BETWEEN_CYCLES + 1)..=(thunder_cycle_end + MAX_TIME_BETWEEN_CYCLES));
            //     thunder_cycle_duration = rng.gen_range(MIN_THUNDER_CYCLE_DURATION..=MAX_THUNDER_CYCLE_DURATION);
            // }

            // start_time = get_thunder_start_time(rain_cycle_start, thunder_cycle_start);
            // if start_time > time {
            //     continue;
            // }
        // }

        // start_time = get_thunder_start_time(rain_cycle_start, thunder_cycle_start);
        // if start_time <= time {
        //     succes += 1;
        // }
    }
    
    succes as f64 / loops as f64
}

pub fn get_first_thunder() -> (u64, u64) {
        let mut rng: StdRng = SeedableRng::from_entropy();
        let mut rain_cycle_start: u64 = rng.gen_range((MIN_TIME_BETWEEN_CYCLES + 1)..MAX_TIME_BETWEEN_CYCLES);
        let mut rain_cycle_duration: u64 = rng.gen_range(MIN_RAIN_CYCLE_DURATION..MAX_RAIN_CYCLE_DURATION);
        let mut thunder_cycle_start: u64 = rng.gen_range((MIN_TIME_BETWEEN_CYCLES + 1)..MAX_TIME_BETWEEN_CYCLES);
        let mut thunder_cycle_duration: u64 = rng.gen_range(MIN_THUNDER_CYCLE_DURATION..MAX_THUNDER_CYCLE_DURATION);

        while get_thunder_duration(
            rain_cycle_start,
            rain_cycle_duration,
            thunder_cycle_start,
            thunder_cycle_duration
        ) == 0 {
            if rain_cycle_start <= thunder_cycle_start {
                let rain_cycle_end: u64 = rain_cycle_start + rain_cycle_duration;
                
                rain_cycle_start = rng.gen_range((rain_cycle_end + MIN_TIME_BETWEEN_CYCLES)..(rain_cycle_end + MAX_TIME_BETWEEN_CYCLES - 1));
                rain_cycle_duration = rng.gen_range(MIN_RAIN_CYCLE_DURATION..MAX_RAIN_CYCLE_DURATION);
            } else {
                let thunder_cycle_end: u64 = thunder_cycle_start + thunder_cycle_duration;
                
                thunder_cycle_start = rng.gen_range((thunder_cycle_end + MIN_TIME_BETWEEN_CYCLES)..(thunder_cycle_end + MAX_TIME_BETWEEN_CYCLES - 1));
                thunder_cycle_duration = rng.gen_range(MIN_THUNDER_CYCLE_DURATION..MAX_THUNDER_CYCLE_DURATION);
            }
        }

        let start_time: u64 = get_thunder_start_time(rain_cycle_start, thunder_cycle_start);
        let duration: u64 = get_thunder_duration(
            rain_cycle_start,
            rain_cycle_duration,
            thunder_cycle_start,
            thunder_cycle_duration
        );

        (start_time, duration)
}

pub fn format_start_time(time: u64) -> String {
    let hours: u64 = time / (ONE_MINUTE_IN_TICKS * 60);
    let minutes: u64 = (time - hours * ONE_MINUTE_IN_TICKS * 60) / ONE_MINUTE_IN_TICKS;
    let seconds: u64 = (time - hours * ONE_MINUTE_IN_TICKS * 60 - minutes * ONE_MINUTE_IN_TICKS) / 20;
    let milliseconds: u64 = (time - hours * ONE_MINUTE_IN_TICKS * 60 - minutes * ONE_MINUTE_IN_TICKS - seconds * 20) * 5;

    let hours: String = if hours.to_string().len() == 1 { format!("0{}", hours) } else { hours.to_string() };
    let minutes: String = if minutes.to_string().len() == 1 { format!("0{}", minutes) } else { minutes.to_string() };
    let seconds: String = if seconds.to_string().len() == 1 { format!("0{}", seconds) } else { seconds.to_string() };
    let milliseconds: u64 = match milliseconds.to_string().len() {
        1 => { milliseconds * 100 },
        2 => { milliseconds * 10 },
        _ => { milliseconds }
    };

    format!("{}:{}:{}.{}", hours, minutes, seconds, milliseconds)
}

pub fn format_duration(time: u64) -> String {
    let hours: u64 = time / (ONE_MINUTE_IN_TICKS * 60);
    let minutes: u64 = (time - hours * ONE_MINUTE_IN_TICKS * 60) / ONE_MINUTE_IN_TICKS;
    let seconds: u64 = (time - hours * ONE_MINUTE_IN_TICKS * 60 - minutes * ONE_MINUTE_IN_TICKS) / 20;
    let ticks: u64 = time - hours * ONE_MINUTE_IN_TICKS * 60 - minutes * ONE_MINUTE_IN_TICKS - seconds * 20;

    let minutes_string: String = match minutes {
        0 => { "".to_owned() }
        1 => { format!("1 minute") }
        _ => { format!("{} minutes", minutes) }
    };
    let seconds_string: String = match seconds {
        0 => { "".to_owned() }
        1 => { format!("1 seconds") }
        _ => { format!("{} seconds", seconds) }
    };
    let ticks_string: String = match ticks {
        0 => { "".to_owned() }
        1 => { format!("1 tick") }
        _ => { format!("{} ticks", ticks) }
    };

    format!("{} {} {}", minutes_string, seconds_string, ticks_string)
}