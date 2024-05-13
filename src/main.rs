use std::cmp::Reverse;

use farkle::Value::*;
use farkle::*;
use rand::rngs::ThreadRng;
use rayon::prelude::*;

fn main() {
    println!("Running Farkle Stats");

    let count = 5_000_000;
    let turns: Vec<usize> = [count; 16]
        .par_iter()
        .map(|x| samples(*x))
        .flatten()
        .collect();

    // println!("{turns:?}");
    for (score, liklihood) in stats(turns) {
        println!("{score}: {liklihood}%");
    }
}

fn samples(count: usize) -> Vec<usize> {
    let mut rng = ThreadRng::default();

    let mut count = count;
    let mut state = StateUnrolled::default();
    let mut turns: Vec<usize> = Vec::with_capacity(count);
    while count > 0 {
        let mut max = 0;
        // one turn
        state = step(state, reserve_some_push_luck, &mut rng);
        while state.score_at_risk > 0 {
            max = state.score_at_risk;
            state = step(state, reserve_all_push_luck, &mut rng);
        }
        turns.push(max);
        count = count - 1;
    }
    turns
}

// strategy
// anything that can be set aside will be set aside, never stops rolling (can't win)
fn reserve_all_push_luck<'a>(state: &StateRolled) -> Vec<Value> {
    state.scorable.clone()
}

// strategy
// sets aside most things but not everything, never stops rolling (can't win)
fn reserve_some_push_luck<'a>(state: &StateRolled) -> Vec<Value> {
    let scorable = state.scorable.clone();

    // take a fresh roll every time it's possible. duh.
    if scorable.len() == state.rolled {
        return scorable;
    }

    // if I don't have to take 222, don't.
    if scorable.clone().into_iter().filter(|x| *x == Two).count() == 3 && scorable.len() > 3 {
        return scorable.into_iter().filter(|x| *x != Two).collect();
    }

    // skip fives if I have more than 3 dice to work with
    let fives = scorable.clone().into_iter().filter(|x| *x == Five).count();
    if fives < 3 && scorable.len() > fives && state.rolled > 3 {
        return scorable.into_iter().filter(|x| *x != Five).collect();
    }

    state.scorable.clone()
}

#[test]
fn test_strategy() {
    // irl found [Five, One, One, Six, Three] -> [One, One, Five] when it should drop that 5.
    let state = StateRolled {
        rolled: 5,
        scorable: vec![One, One, Five],
        score_at_risk: 100,
        score: 0,
    };

    let reserved = reserve_some_push_luck(&state);
    assert_eq!(vec![One, One], reserved)
}

fn stats(results: Vec<usize>) -> Vec<(usize, f64)> {
    let mut results: Vec<usize> = results;
    results.par_sort_unstable_by_key(|w| Reverse(*w));
    let total = results.len();

    let mut output: Vec<(usize, f64)> = vec![];

    for thresh in [
        50, 300, 500, 750, 1000, 1500, 2000, 2500, 3000, 3500, 4000, 4500, 5000, 5500, 6000, 6500,
        7500, 8500, 9450, 9500, 10000,
    ] {
        // todo not the most effecient way to do this
        let count = results.iter().take_while(|x| **x >= thresh).count();
        output.push((thresh, count as f64 / total as f64 * 100f64));
    }

    output
}
