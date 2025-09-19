use crate::game::{GameState, Mancala, Move, Player};
use std::cell::Cell;
use std::time::{Duration, Instant};

pub(crate) type StateEvalFn<const N: usize> = fn(&GameState<N>, player: Player) -> f32;

#[derive(Debug, Clone)]
pub(crate) struct Minimax<const N: usize> {
    optimize_for: Player,
    max_depth: Option<usize>,
    max_time: Option<Duration>,
    iterative_deepening: bool,
    evaluator: StateEvalFn<N>,
    heuristic: StateEvalFn<N>,
    current_depth: Cell<usize>,
    start_time: Cell<Option<Instant>>,
}

#[derive(Debug, Clone)]
pub(crate) struct MinimaxBuilder<const N: usize> {
    optimize_for: Player,
    max_depth: Option<usize>,
    max_time: Option<Duration>,
    iterative_deepening: bool,
    evaluator: StateEvalFn<N>,
    heuristic: StateEvalFn<N>,
}

impl<const N: usize> MinimaxBuilder<N> {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }
    pub(crate) fn max_time(mut self, time: Duration) -> Self {
        self.max_time = Some(time);
        self
    }
    pub(crate) fn iterative_deepening(mut self, iterative: bool) -> Self {
        self.iterative_deepening = iterative;
        self
    }
    pub(crate) fn evaluator(mut self, e: StateEvalFn<N>) -> Self {
        self.evaluator = e;
        self
    }
    pub(crate) fn heuristic(mut self, h: StateEvalFn<N>) -> Self {
        self.heuristic = h;
        self
    }
    pub(crate) fn build(&self) -> Minimax<N> {
        Minimax {
            optimize_for: self.optimize_for,
            max_depth: self.max_depth,
            max_time: self.max_time,
            iterative_deepening: self.iterative_deepening,
            evaluator: self.evaluator,
            heuristic: self.heuristic,
            current_depth: 0.into(),
            start_time: None.into(),
        }
    }
}

impl MinimaxBuilder<6> {
    pub(crate) fn new_standard() -> Self {
        Self::default()
    }
}

impl<const N: usize> Default for MinimaxBuilder<N> {
    fn default() -> Self {
        let evaluator = |s: &GameState<N>, p: Player| match p {
            Player::One => (s.score(Player::One) - s.score(Player::Two)) as f32,
            Player::Two => (s.score(Player::Two) - s.score(Player::One)) as f32,
        };
        let heuristic = evaluator;
        Self {
            optimize_for: Player::One,
            max_depth: Some(12),
            max_time: None,
            iterative_deepening: false,
            evaluator,
            heuristic,
        }
    }
}

impl<const N: usize> Minimax<N> {
    pub(crate) fn optimize_for(&self) -> Player {
        self.optimize_for
    }
    pub(crate) fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }
    pub(crate) fn max_time(&self) -> Option<Duration> {
        self.max_time
    }
    pub(crate) fn iterative_deepening(&self) -> bool {
        self.iterative_deepening
    }
    pub(crate) fn current_depth(&self) -> usize {
        self.current_depth.get()
    }
    pub(crate) fn start_time(&self) -> Option<Instant> {
        self.start_time.get()
    }
    pub(crate) fn call_evaluator(&self, state: &GameState<N>) -> f32 {
        (self.evaluator)(state, self.optimize_for)
    }
    pub(crate) fn call_heuristic(&self, state: &GameState<N>) -> f32 {
        (self.heuristic)(state, self.optimize_for)
    }

    pub(crate) fn search(&self, state: &GameState<N>) -> Option<Move> {
        self.start_time.set(Some(Instant::now()));
        self.current_depth.set(0);

        // TODO: Implement iterative deepening.

        let (best_move, _) = self.max_value(state, f32::NEG_INFINITY, f32::INFINITY);

        self.start_time.set(None);
        self.current_depth.set(0);
        best_move
    }

    fn time_exceeded(&self) -> bool {
        match (self.start_time(), self.max_time) {
            (Some(start), Some(max)) => Instant::now() - start >= max,
            _ => false,
        }
    }

    fn max_value(&self, state: &GameState<N>, alpha: f32, beta: f32) -> (Option<Move>, f32) {
        assert_ne!(
            self.start_time.get(),
            None,
            "Minimax search must be started with `search()` before calling `min_value`"
        );

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            return (None, self.call_evaluator(state));
        }

        // If we have reached the artificial limit, use the heuristic.
        if self.max_depth.is_some_and(|d| self.current_depth() >= d) || self.time_exceeded() {
            return (None, self.call_heuristic(state));
        }

        self.current_depth.set(self.current_depth() + 1);
        let mut alpha = alpha;
        let mut v = f32::NEG_INFINITY;
        let mut best_move: Option<Move> = None;

        for m in state.valid_moves() {
            let new_state = state.make_move(m).unwrap();
            let (_, v2) = if new_state.current_turn() == state.current_turn() {
                self.max_value(&new_state, alpha, beta)
            } else {
                self.min_value(&new_state, alpha, beta)
            };

            if v2 > v {
                v = v2;
                best_move = Some(m);
                alpha = if alpha > v { alpha } else { v };
            }

            // Alpha > beta ==> prune
            if v >= beta {
                return (best_move, v);
            }
        }

        (best_move, v)
    }

    fn min_value(&self, state: &GameState<N>, alpha: f32, beta: f32) -> (Option<Move>, f32) {
        assert_ne!(
            self.start_time(),
            None,
            "Minimax search must be started with `search()` before calling `min_value`"
        );

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            return (None, self.call_evaluator(state));
        }

        // If we have reached the artificial limit, use the heuristic.
        if self.max_depth.is_some_and(|d| self.current_depth() >= d) || self.time_exceeded() {
            return (None, self.call_heuristic(state));
        }

        self.current_depth.set(self.current_depth() + 1);
        let mut beta = beta;
        let mut v = f32::INFINITY;
        let mut best_move: Option<Move> = None;

        for m in state.valid_moves() {
            let new_state = state.make_move(m).unwrap();
            let (_, v2) = if new_state.current_turn() == state.current_turn() {
                self.min_value(&new_state, alpha, beta)
            } else {
                self.max_value(&new_state, alpha, beta)
            };

            if v2 < v {
                v = v2;
                best_move = Some(m);
                beta = if beta < v { beta } else { v };
            }

            // Alpha > beta ==> prune
            if v <= alpha {
                return (best_move, v);
            }
        }

        (best_move, v)
    }
}
