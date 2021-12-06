use dashmap::DashMap;
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use tracing::warn;

/// A struct representing a past contest event.
#[derive(Debug, Clone)]
pub struct PlayerEvent {
    contest_name: String,
    perf: f64,
    rating: f64,
    contest_place: usize,
    rating_place: usize,
}

impl PlayerEvent {
    fn new(
        contest_name: String,
        perf: f64,
        rating: f64,
        contest_place: usize,
        rating_place: usize,
    ) -> Self {
        Self {
            contest_name,
            perf,
            rating,
            contest_place,
            rating_place,
        }
    }

    /// Get a reference to the player event's contest name.
    pub fn contest_name(&self) -> &str {
        self.contest_name.as_ref()
    }

    /// Get a reference to the player event's perf.
    pub fn perf(&self) -> f64 {
        self.perf
    }

    /// Get a reference to the player event's rating.
    pub fn rating(&self) -> f64 {
        self.rating
    }

    /// Get a reference to the player event's contest place.
    pub fn contest_place(&self) -> usize {
        self.contest_place
    }

    /// Get a reference to the player event's rating place.
    pub fn rating_place(&self) -> usize {
        self.rating_place
    }
}

#[derive(Debug, Clone)]
struct Player {
    mu: f64,
    mu_pi: f64,
    sigma: f64,
    delta: f64,

    perfs: Vec<f64>,
    weights: Vec<f64>,

    history: Vec<PlayerEvent>,
}

impl Player {
    fn new(mu: f64, sigma: f64) -> Player {
        Player {
            mu,
            mu_pi: 0.0,
            sigma,
            delta: 0.0,
            perfs: vec![mu],
            weights: vec![sigma.powi(-2)],
            history: vec![PlayerEvent::new("-".to_string(), mu, mu, 0, 0)],
        }
    }

    fn diffuse(&mut self, rho: f64, gamma: f64) {
        let kappa = (1.0 + (gamma.powi(2) / self.sigma.powi(2))).recip();
        let mut kappa_rho = kappa.powf(rho);
        let w_g = kappa_rho * self.weights[0];
        let w_l = (1.0 - kappa_rho) * self.weights.iter().sum::<f64>();
        self.perfs[0] = (w_g * self.perfs[0] + w_l * self.mu) / (w_g + w_l);
        self.weights[0] = kappa * (w_g + w_l);
        kappa_rho *= kappa;
        for w in self.weights.iter_mut().skip(1) {
            *w *= kappa_rho;
        }
        self.sigma /= kappa.sqrt();
    }

    fn update(&mut self, beta: f64, player_data: &[(f64, f64)], (lo, hi): (usize, usize)) {
        // COEFF = PI / sqrt(3)
        const COEFF: f64 = 1.8137993642342178;
        const SOLVE_BOUND: (f64, f64) = (-10000.0, 10000.0);

        let f = |x: f64| {
            let mut result = 0.0;
            for &(delta, mu_pi) in player_data.iter().skip(lo - 1) {
                result += delta.recip() * ((COEFF * (x - mu_pi) / (2.0 * delta)).tanh() - 1.0);
            }
            for &(delta, mu_pi) in player_data.iter().take(hi) {
                result += delta.recip() * ((COEFF * (x - mu_pi) / (2.0 * delta)).tanh() + 1.0);
            }
            result
        };

        self.perfs.push(solve_itp(SOLVE_BOUND, f));
        self.weights.push(beta.powi(-2));

        let f = |x: f64| {
            let mut result = 0.0;
            result += self.weights[0] * (x - self.perfs[0]);
            for k in 1..self.perfs.len() {
                result += (COEFF * beta * self.weights[k])
                    * (COEFF * (x - self.perfs[k]) / (2.0 * beta)).tanh();
            }
            result
        };

        self.mu = solve_itp(SOLVE_BOUND, f);
    }
}

/// A wrapper of contest results.
#[derive(Debug, Clone)]
pub struct Contest {
    name: String,
    standings: Vec<(String, i64)>,
}

impl Contest {
    /// Construct a new contest.
    /// 
    /// The first element of standings is a player's identifier.
    /// 
    /// The second element is the score (higher is better).
    pub fn new(name: String, standings: Vec<(String, i64)>) -> Self {
        Self { name, standings }
    }
}

/// An implementation of EloMMR algorithm.
#[derive(Debug, Clone)]
pub struct EloMmr {
    rho: f64,
    beta: f64,
    gamma: f64,
    mu_init: f64,
    sigma_init: f64,

    players: DashMap<String, Player>,
}

impl Default for EloMmr {
    fn default() -> Self {
        Self::new(1.0, 200.0, 80.0, 1500.0, 350.0)
    }
}

impl EloMmr {
    /// Construct a new system.
    /// 
    /// Default::default() gives a preset of superparameters (ρ = 1, β = 200, γ = 80, μ_init = 1500, σ_init = 350).
    pub fn new(rho: f64, beta: f64, gamma: f64, mu_init: f64, sigma_init: f64) -> EloMmr {
        EloMmr {
            rho,
            beta,
            gamma,
            mu_init,
            sigma_init,
            players: DashMap::new(),
        }
    }

    /// Update ratings according to the result of the provided contest.
    pub fn update(&self, contest: &mut Contest) {
        // Every line of codes after this assumes that contest.standings is not empty.
        if contest.standings.is_empty() {
            return;
        }

        let standings = calc_standings(&mut contest.standings);

        let mut player_datas = Vec::with_capacity(standings.len());
        standings
            .par_iter()
            .map(|(id, _, _)| {
                let mut player = self
                    .players
                    .entry(id.clone())
                    .or_insert_with(|| Player::new(self.mu_init, self.sigma_init));
                player.diffuse(self.rho, self.gamma);
                player.mu_pi = player.mu;
                player.delta = player.sigma.hypot(self.beta);
                (player.delta, player.mu_pi)
            })
            .collect_into_vec(&mut player_datas);

        standings.par_iter().for_each(|(id, lo, hi)| {
            let mut player = self.players.get_mut(id).unwrap();
            player.update(self.beta, &player_datas, (*lo, *hi));
            let new_history = PlayerEvent::new(
                contest.name.clone(),
                *player.perfs.last().unwrap(),
                player.mu,
                *lo,
                0,
            );
            player.history.push(new_history);
        });

        // Rank players according to new ratings.
        let mut new_ratings = self.export_ratings();
        new_ratings.par_sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
        let mut rank_app = 0;
        let mut rank_int = 0;
        for (i, (id, score)) in new_ratings.iter().enumerate() {
            rank_int += 1;
            if i == 0 || *score != new_ratings[i - 1].1 {
                rank_app = rank_int;
            }
            let mut player = self.players.get_mut(id).unwrap();
            let last_event = player.history.last_mut().unwrap();
            if last_event.rating_place == 0 {
                last_event.rating_place = rank_app;
            }
        }
    }

    /// Export all players' rating.
    /// 
    /// The returned String is the id of the player.
    pub fn export_ratings(&self) -> Vec<(String, f64)> {
        self.players
            .par_iter()
            .map(|player| (player.key().clone(), player.mu))
            .collect()
    }

    /// Export all players' history.
    /// 
    /// The returned String is the id of the player.
    pub fn export_history(&self) -> Vec<(String, Vec<PlayerEvent>)> {
        self.players
            .par_iter()
            .map(|player| (player.key().clone(), player.history.clone()))
            .collect()
    }

    /// Export a player's history.
    pub fn export_history_of(&self, id: &str) -> Option<Vec<PlayerEvent>> {
        self.players.get(id).map(|player| player.history.clone())
    }
}

/// Solve f(x) = 0 where x belongs to [a, b].
/// 
/// May return inaccurate solutions if y_a * y_b > 0.
fn solve_itp((mut a, mut b): (f64, f64), mut f: impl FnMut(f64) -> f64) -> f64 {
    const EPSILON: f64 = 1e-10;
    const N_0: usize = 1;

    debug_assert!(a < b);

    let mut y_a = f(a);
    let mut y_b = f(b);

    if y_a * y_b > 0.0 {
        panic!("not bracketing a zero point");
    }

    if y_a > y_b {
        panic!("not satisfying y_a < y_b condition");
    }

    let n_half = (((b - a) / EPSILON).log2().ceil() - 1.0).max(0.0) as usize;
    let n_max = n_half + N_0;
    let k_1 = 0.2 / (b - a);

    let mut scaled_epsilon = EPSILON * (1u64 << n_max) as f64;

    while b - a > 2.0 * EPSILON {
        let x_half = 0.5 * (a + b);
        let r = scaled_epsilon - 0.5 * (b - a);
        let x_f = (y_b * a - y_a * b) / (y_b - y_a);
        let sigma = x_half - x_f;
        let delta = k_1 * (b - a).powi(2);
        let x_t = if delta <= sigma.abs() {
            x_f + delta.copysign(sigma)
        } else {
            x_half
        };
        let x_itp = if (x_t - x_half).abs() <= r {
            x_t
        } else {
            x_half - r.copysign(sigma)
        };
        let y_itp = f(x_itp);
        if y_itp > 0.0 {
            b = x_itp;
            y_b = y_itp;
        } else if y_itp < 0.0 {
            a = x_itp;
            y_a = y_itp;
        } else {
            return x_itp;
        }
        scaled_epsilon *= 0.5;
    }

    (a + b) * 0.5
}

/// Calc (lo, hi) standing data used in EloMMR.
/// 
/// Assume that raw is **not** empty.
fn calc_standings(raw: &mut Vec<(String, i64)>) -> Vec<(String, usize, usize)> {
    raw.par_sort_unstable_by_key(|v| -v.1);
    let mut standings: Vec<(String, usize, usize)> = Vec::new();
    let mut rank_app = 1;
    let mut rank_int = 1;
    standings.push((raw[0].0.to_owned(), 1, 0));
    for (i, (id, score)) in raw.iter().enumerate().skip(1) {
        rank_int += 1;
        if *score != raw[i - 1].1 {
            rank_app = rank_int;
        }
        standings.push((id.to_owned(), rank_app, 0));
    }
    standings.last_mut().unwrap().2 = rank_app;
    for (i, (_, score)) in raw.iter().enumerate().rev().skip(1) {
        rank_int -= 1;
        if *score != raw[i + 1].1 {
            rank_app = rank_int;
        }
        standings[i].2 = rank_app;
    }
    standings
}
