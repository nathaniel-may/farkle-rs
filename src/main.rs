use farkle::*;

fn main() {
    println!("Running Farkle Stats");
    let mut state = StateUnrolled::default();

    let mut count = 1_000_000;
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

    for (score, liklihood) in stats(&turns) {
        println!("{score}: {liklihood}%");
    }
}

// strategy
// anything that can be set aside will be set aside, never stops rolling (can't win)
fn reserve_all_push_luck<'a>(state: &StateRolled) -> Vec<Value> {
    todo!()
}

fn stats(results: &[usize]) -> Vec<(usize, f32)> {
    todo!()
}
