use rand::prelude::*;

pub fn get_results(request: &str) -> Vec<i32> {
    let should_explode = request.starts_with("!");

    let s = if should_explode {
        &request[1..]
    } else {
        request
    };

    let dice_result: Vec<i32> = s.split('+')
        .map(|part| run(part, should_explode))
        .filter_map(|x| x)
        .collect();

    if dice_result.len() == 0 {
        vec![-666]
    } else {
        dice_result
    }
}

fn run(input: &str, should_explode: bool) -> Option<i32> {
    if input.contains('d') {
        let parts: Vec<&str> = input.split('d').collect();
        let a = if let Ok(i) = parts[0].parse::<u32>() {
            i
        } else {
            return None;
        };
        let b = if let Ok(i) = parts[1].parse::<u32>() {
            i
        } else {
            return None;
        };

        return Some(roll(a, b, should_explode));
    }

    if let Ok(i) = input.parse::<i32>() {
        Some(i)
    } else {
        None
    }
}

fn roll(a: u32, b: u32, should_explode: bool) -> i32 {
    (1..=a).map(|_| roll_single(b, should_explode)).sum()
}

fn roll_single(x: u32, should_explode: bool) -> i32 {
    let mut rng = thread_rng();
    let mut result: i32 = 0;
    loop {
        let i: u32 = rng.gen_range(1, x + 1);
        result += i as i32;
        if !should_explode || (i < x && x > 1) || x == 1 {
            break;
        }
    }
    result
}
