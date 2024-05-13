use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

use Value::*;

pub type Roll = dyn Fn() -> Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateUnrolled {
    pub dice_left: usize,
    pub score_at_risk: usize,
    pub score: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateRolled {
    pub scorable: Vec<Value>,
    pub score_at_risk: usize,
    pub score: usize,
}

impl Default for StateUnrolled {
    fn default() -> Self {
        StateUnrolled {
            dice_left: 6,
            score_at_risk: 0,
            score: 0,
        }
    }
}

pub fn step<'a, F>(state: StateUnrolled, strategy: F) -> StateUnrolled
where
    F: Fn(&StateRolled) -> Vec<Value>,
{
    fn f() -> Value {
        let mut rng = thread_rng();
        let dist = Uniform::new(1, 6);
        match rng.sample(dist) {
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            _ => Six,
        }
    }
    _step(state, strategy, f)
}

fn _step<'a, F, R>(state: StateUnrolled, strategy: F, roll: R) -> StateUnrolled
where
    F: Fn(&StateRolled) -> Vec<Value>,
    R: Fn() -> Value,
{
    let mut rolled: Vec<Value> = vec![];
    for _ in 1..=state.dice_left {
        rolled.push(roll());
    }

    let (max_score, scorable) = score(&rolled);
    let farkle = max_score == 0;

    if farkle {
        return StateUnrolled {
            dice_left: 6,
            score_at_risk: 0,
            score: state.score,
        };
    }

    // state the "player" sees to apply their strategy to
    let state = StateRolled {
        scorable,
        score_at_risk: state.score_at_risk + max_score,
        score: state.score,
    };
    match &strategy(&state)[..] {
        // strategy didn't reserve any dice, so they cannot continue rolling.
        // tally the score_at_risk and move on
        [] => StateUnrolled {
            dice_left: 6,
            score_at_risk: 0,
            score: state.score + state.score_at_risk,
        },

        // legality of the provided strategy is not actually enforced
        reserved => {
            let (score, _) = score(reserved);
            StateUnrolled {
                dice_left: state.scorable.len() - reserved.len(),
                score_at_risk: state.score_at_risk + score,
                score: state.score,
            }
        }
    }
}

#[test]
fn test_step() {
    let next = _step(StateUnrolled::default(), |st| st.scorable.clone(), || One);
    assert_eq!(0, next.dice_left);
    assert_eq!(6000, next.score_at_risk);
}

// used to score six OR LESS dice
pub fn score<'a>(dice: &[Value]) -> (usize, Vec<Value>) {
    fn score(m: &mut HashMap<usize, Vec<Value>>, scored: &mut Vec<Value>) -> usize {
        // base case
        if m.is_empty() {
            return 0;
        }

        // 6 of a kind
        if let Some(values) = m.get(&6) {
            scored.extend(vec![values[0]; 6]);
            return 3000;
        }

        // two triples
        if let Some(2) = m.get(&3).map(|xs| xs.len()) {
            scored.extend(m.get(&3).unwrap());
            scored.extend(m.get(&3).unwrap());
            scored.extend(m.get(&3).unwrap());
            return 2500;
        }

        // straight
        if let Some(6) = m.get(&1).map(|xs| xs.len()) {
            scored.extend(m.get(&1).unwrap());
            return 2500;
        }

        // 5 of a kind
        if let Some(values) = m.get(&5) {
            scored.extend(vec![values[0]; 5]);
            m.remove(&5);
            // the last die could be a 1 or a 5
            return 2000 + score(m, scored);
        }

        // three pairs that are different
        if let Some(3) = m.get(&2).map(|xs| xs.len()) {
            scored.extend(m.get(&2).unwrap());
            scored.extend(m.get(&2).unwrap());
            return 1500;
        }

        // four of a kind and one pair = three pairs
        if m.get(&4).is_some() && m.get(&2).is_some() {
            scored.extend(m.get(&4).unwrap());
            scored.extend(m.get(&2).unwrap());
            return 1500;
        }

        // four of a kind that is not three pairs
        if let Some(values) = m.get(&4) {
            scored.extend(vec![values[0]; 4]);
            m.remove(&4);
            return 2000 + score(m, scored);
        }

        // 3 of a kind
        if let Some(values) = m.get(&3) {
            // more than one triple is caught earlier so this is safe
            let triple_value = match values[0] {
                One | Three => 300,
                Two => 200,
                Four => 400,
                Five => 500,
                Six => 600,
            };
            scored.extend(vec![values[0]; 3]);
            m.remove(&3);
            // the last die could be a 1 or a 5
            return triple_value + score(m, scored);
        }

        // TODO I can probably figure out how to get rid of this clone
        let ones = m.clone().into_iter().find_map(|(count, values)| {
            if values.contains(&One) {
                Some(count)
            } else {
                None
            }
        });

        if let Some(ones) = ones {
            scored.extend(vec![One; ones]);
            m.retain(|_, values| !values.contains(&One));
            return ones * 100 + score(m, scored);
        }

        // TODO I can probably figure out how to get rid of this clone
        let fives = m.clone().into_iter().find_map(|(count, values)| {
            if values.contains(&Five) {
                Some(count)
            } else {
                None
            }
        });

        if let Some(fives) = fives {
            scored.extend(vec![Five; fives]);
            m.retain(|_, values| !values.contains(&Five));
            return fives * 50 + score(m, scored);
        }

        0
    }

    let count = |v: Value| dice.iter().filter(|&vv| *vv == v).count();
    let mut m = HashMap::new();
    for value in [One, Two, Three, Four, Five, Six] {
        let count = count(value);
        // TODO there's probably a cleaner way to write this line
        let mut values: Vec<Value> = m.get(&count).unwrap_or(&vec![]).iter().cloned().collect();
        values.push(value);
        m.insert(count, values);
    }

    let mut scored = vec![];

    (score(&mut m, &mut scored), scored)
}

#[test]
fn test_scores() {
    assert_eq!(3000, score(&[One, One, One, One, One, One]).0);
    assert_eq!(2500, score(&[One, Two, Three, Four, Five, Six]).0);
    assert_eq!(2050, score(&[One, One, One, One, One, Five]).0);
    assert_eq!(2500, score(&[One, One, One, Two, Two, Two]).0);
    assert_eq!(1500, score(&[One, One, Two, Two, Two, Two]).0);
    assert_eq!(
        (200, vec![One, Five, Five]),
        score(&[Five, Five, One, Two, Three, Two])
    );
    assert_eq!(300, score(&[One, One, One]).0);
    assert_eq!(0, score(&[]).0);
    assert_eq!((50, vec![Five]), score(&[Two, Five]));
}
