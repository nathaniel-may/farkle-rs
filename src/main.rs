use farkle::*;

fn main() {
    println!("Running Farkle Stats");
    let mut state = StateUnrolled::default();

    let mut count = 100000;
    let mut turns: Vec<usize> = Vec::with_capacity(count);
    while count > 0 {
        let mut max = 0;
        // one turn
        state = step(state, reserve_all_push_luck);
        while state.score_at_risk > 0 {
            max = state.score_at_risk;
            state = step(state, reserve_all_push_luck);
        }
        turns.push(max);
        count = count - 1;
    }

    for (score, liklihood) in stats(turns) {
        println!("{score}: {liklihood}%");
    }
}

// strategy
// anything that can be set aside will be set aside, never stops rolling (can't win)
fn reserve_all_push_luck<'a>(state: &StateRolled) -> Vec<Value> {
    state.scorable.clone()
}

fn stats(results: Vec<usize>) -> Vec<(usize, f32)> {
    let mut results: Vec<usize> = results;
    results.sort_unstable();
    let total = results.len();

    let mut output: Vec<(usize, f32)> = vec![];

    for thresh in [
        300, 500, 750, 1000, 1500, 2000, 2500, 3000, 3500, 4000, 4500, 5000, 5500, 6000, 6500,
        7500, 8500, 9450, 9500, 10000,
    ] {
        // todo not the most effecient way to do this
        let count = results.iter().take_while(|x| **x <= thresh).count();
        output.push((thresh, count as f32 / total as f32 * 100f32));
    }

    output
}
