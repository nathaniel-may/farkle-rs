use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
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
    pub rolled: usize,
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

fn roll(rng: &mut ThreadRng) -> Value {
    // [low, high)
    let dist = Uniform::new(1, 7);
    match rng.sample(dist) {
        1 => One,
        2 => Two,
        3 => Three,
        4 => Four,
        5 => Five,
        6 => Six,
        _ => panic!("that's not how dice work."),
    }
}

pub fn step<'a, F>(state: StateUnrolled, strategy: F, rng: &mut ThreadRng) -> StateUnrolled
where
    F: Fn(&StateRolled) -> Vec<Value>,
{
    let mut rolled: Vec<Value> = vec![];
    for _ in 1..=state.dice_left {
        rolled.push(roll(rng));
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
        rolled: state.dice_left,
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
            let mut dice_left = state.rolled - reserved.len();
            if dice_left == 0 {
                dice_left = 6;
            }
            StateUnrolled {
                dice_left,
                score_at_risk: state.score_at_risk - max_score + score,
                score: state.score,
            }
        }
    }
}

// used to score six OR LESS dice
pub fn score<'a>(dice: &[Value]) -> (usize, Vec<Value>) {
    fn score(m: &mut HashMap<usize, Vec<Value>>, scored: &mut Vec<Value>) -> usize {
        // the map should have everything in it all the time.
        assert!(m.values().flatten().count() == 6);

        // base case (complicated way of representing an empty state)
        if let Some(values) = m.get(&0) {
            if values.len() == 6 {
                return 0;
            }
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
            let mut next = m.get(&0).cloned().unwrap_or(vec![]);
            next.extend(m.get(&5).unwrap());
            m.insert(0, next);
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
            let values = values.clone();
            scored.extend(vec![values[0]; 4]);
            m.remove(&4);
            let mut next = m.get(&0).cloned().unwrap_or(vec![]);
            next.push(values[0]);
            m.insert(0, next);
            return 2000 + score(m, scored);
        }

        // 3 of a kind
        if let Some(values) = m.get(&3) {
            let values = values.clone();
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
            let mut next = m.get(&0).cloned().unwrap_or(vec![]);
            next.extend(values);
            m.insert(0, next);
            // the last die could be a 1 or a 5
            return triple_value + score(m, scored);
        }

        // TODO I can probably figure out how to get rid of this clone
        let ones = m.clone().into_iter().find_map(|(count, values)| {
            if values.contains(&One) && count > 0 {
                Some(count)
            } else {
                None
            }
        });

        if let Some(ones) = ones {
            scored.extend(vec![One; ones]);
            m.get_mut(&ones).unwrap().retain(|x| *x != One);
            let mut next = m.get(&0).cloned().unwrap_or(vec![]);
            next.push(One);
            m.insert(0, next);
            return ones * 100 + score(m, scored);
        }

        // TODO I can probably figure out how to get rid of this clone
        let fives = m.clone().into_iter().find_map(|(count, values)| {
            if values.contains(&Five) && count > 0 {
                Some(count)
            } else {
                None
            }
        });

        if let Some(fives) = fives {
            scored.extend(vec![Five; fives]);
            m.get_mut(&fives).unwrap().retain(|x| *x != Five);
            let mut next = m.get(&0).cloned().unwrap_or(vec![]);
            next.push(Five);
            m.insert(0, next);
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

    // minified of below
    assert_eq!((150, vec![One, Five]), score(&[One, Five]));

    // found by making real runs:
    assert_eq!(
        (300, vec![One, One, Five, Five]),
        score(&[Six, One, Five, One, Four, Five])
    );
}
