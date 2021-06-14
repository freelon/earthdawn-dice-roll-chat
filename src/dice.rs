use std::collections::VecDeque;

use rand::prelude::*;

pub fn get_results(request: &str) -> Vec<i32> {
    let plus_minus: &[char] = &['+', '-'][..];

    let request = strip_text(request);
    let should_explode = request.starts_with("!");

    let s = if should_explode {
        &request[1..]
    } else {
        request
    };

    let mut separators: VecDeque<&str> = s.matches(plus_minus).collect();
    if !s.starts_with(plus_minus) {
        separators.push_front("+");
    }

    let dice_result: Vec<i32> = s
        .split(plus_minus)
        .zip(separators)
        .map(|(part, sign)| run(&format!("{}{}", sign, part), should_explode))
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
        let negative = input.starts_with('-');

        let input = if negative { &input[1..] } else { &input[..] };

        let parts: Vec<&str> = input.split('d').collect();
        let a = if parts[0].is_empty() {
            1
        } else {
            if let Ok(i) = parts[0].parse::<u32>() {
                i
            } else {
                return None;
            }
        };
        let b = if let Ok(i) = parts[1].parse::<u32>() {
            i
        } else {
            return None;
        };

        let mut result = roll(a, b, should_explode);
        if negative {
            result = -result;
        }

        return Some(result);
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
    if x < 1 {
        return 0;
    }

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

fn strip_text(input: &str) -> &str {
    let roll_part = if input.contains(" ") {
        input.split(" ").next().unwrap()
    } else {
        input
    };

    if roll_part.ends_with('*') {
        &roll_part[..roll_part.len() - 1]
    } else {
        roll_part
    }
}

pub fn is_hidden_roll(input: &str) -> bool {
    let roll_part = if input.contains(" ") {
        input.split_once(" ").unwrap().0
    } else {
        input
    };

    roll_part.ends_with('*')
}

pub fn hide_roll_part(input: &str) -> String {
    let text_part = if input.contains(" ") {
        input.split_once(" ").unwrap().1
    } else {
        ""
    };

    format!("!*hidden* {}", text_part)
}

#[cfg(test)]
mod test {
    use super::get_results;

    #[test]
    fn test_number() {
        assert_eq!(vec![1, 2, 3], get_results(&"1+2+3"));
    }

    #[test]
    fn test_negative_with_number() {
        assert_eq!(vec![1, 2, -3], get_results(&"1+2+-3"));
    }

    #[test]
    fn test_negative_with_dice() {
        assert_eq!(vec![1, 2, -3], get_results(&"1d1+2d1-3d1"));
    }

    #[test]
    fn test_dice() {
        assert_eq!(vec![1, 2, 3], get_results(&"1d1+2d1+3d1"));
    }
}
