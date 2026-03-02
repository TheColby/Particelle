use std::f64::consts::PI;
use super::schema::WindowSpec;

/// Modified Bessel function of the first kind (order zero).
/// Used in the Kaiser window implementation.
fn besseli0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut u = 1.0;
    let half_x = x / 2.0;
    let mut n = 1.0;
    
    // Series expansion: I0(x) = sum_{k=0}^inf ( (x/2)^k / k! )^2
    while u > 1e-15 {
        let temp = half_x / n;
        u *= temp * temp;
        sum += u;
        n += 1.0;
    }
    
    sum
}

/// Generate a complete window of the given specification and length.
/// Length must be > 0.
pub fn generate(spec: &WindowSpec, len: usize) -> Vec<f64> {
    if len == 0 {
        return vec![];
    }
    
    let mut out = vec![0.0; len];
    if len == 1 {
        out[0] = 1.0;
        return out;
    }

    // Pass through composite logic if needed
    match spec {
        WindowSpec::Asymmetric { left, right } => {
            let mid = len / 2;
            let left_win = generate(left, mid * 2);
            let right_win = generate(right, mid * 2);
            for i in 0..mid {
                out[i] = left_win[i];
                out[mid + i] = right_win[mid + i];
            }
            if len % 2 != 0 {
                out[len - 1] = right_win[mid * 2 - 1]; // Handle center point logic
            }
            return out;
        }
        WindowSpec::Symmetric { base } => {
            let base_win = generate(base, len); // Most are symmetric anyway, this is for forcing
            return base_win;
        }
        WindowSpec::HalfLeft { base } => {
            let base_win = generate(base, len * 2 - 1);
            out.copy_from_slice(&base_win[0..len]);
            return out;
        }
        WindowSpec::HalfRight { base } => {
            let base_win = generate(base, len * 2 - 1);
            out.copy_from_slice(&base_win[(len - 1)..]);
            return out;
        }
        _ => {}
    }

    let n_f64 = (len - 1) as f64;

    for i in 0..len {
        let n = i as f64;
        let v = match spec {
            WindowSpec::Rectangular => 1.0,
            WindowSpec::Hann => 0.5 * (1.0 - (2.0 * PI * n / n_f64).cos()),
            WindowSpec::Hamming => 0.54 - 0.46 * (2.0 * PI * n / n_f64).cos(),
            WindowSpec::Blackman => {
                0.42 - 0.5 * (2.0 * PI * n / n_f64).cos() + 0.08 * (4.0 * PI * n / n_f64).cos()
            }
            WindowSpec::BlackmanHarris => {
                let a0 = 0.35875;
                let a1 = 0.48829;
                let a2 = 0.14128;
                let a3 = 0.01168;
                a0 - a1 * (2.0 * PI * n / n_f64).cos()
                    + a2 * (4.0 * PI * n / n_f64).cos()
                    - a3 * (6.0 * PI * n / n_f64).cos()
            }
            WindowSpec::Nuttall => {
                let a0 = 0.355768;
                let a1 = 0.487396;
                let a2 = 0.144232;
                let a3 = 0.012604;
                a0 - a1 * (2.0 * PI * n / n_f64).cos()
                    + a2 * (4.0 * PI * n / n_f64).cos()
                    - a3 * (6.0 * PI * n / n_f64).cos()
            }
            WindowSpec::BlackmanNuttall => {
                let a0 = 0.3635819;
                let a1 = 0.4891775;
                let a2 = 0.1365995;
                let a3 = 0.0106411;
                a0 - a1 * (2.0 * PI * n / n_f64).cos()
                    + a2 * (4.0 * PI * n / n_f64).cos()
                    - a3 * (6.0 * PI * n / n_f64).cos()
            }
            WindowSpec::FlatTop => {
                let a0 = 0.21557895;
                let a1 = 0.41663158;
                let a2 = 0.27726316;
                let a3 = 0.08357895;
                let a4 = 0.00694737;
                a0 - a1 * (2.0 * PI * n / n_f64).cos()
                    + a2 * (4.0 * PI * n / n_f64).cos()
                    - a3 * (6.0 * PI * n / n_f64).cos()
                    + a4 * (8.0 * PI * n / n_f64).cos()
            }
            WindowSpec::Bartlett => {
                2.0 / n_f64 * (n_f64 / 2.0 - (n - n_f64 / 2.0).abs())
            }
            WindowSpec::BartlettHann => {
                let a0 = 0.62;
                let a1 = 0.48;
                let a2 = 0.38;
                a0 - a1 * (n / n_f64 - 0.5).abs() - a2 * (2.0 * PI * n / n_f64).cos()
            }
            WindowSpec::Bohman => {
                let t = (n - n_f64 / 2.0).abs() / (n_f64 / 2.0);
                if t <= 1.0 {
                    (1.0 - t) * (PI * t).cos() + (1.0 / PI) * (PI * t).sin()
                } else {
                    0.0
                }
            }
            WindowSpec::Cosine => {
                (PI * n / n_f64).sin()
            }
            WindowSpec::Sine => {
                (PI * n / n_f64).sin()
            }
            WindowSpec::Lanczos => {
                let t = 2.0 * n / n_f64 - 1.0;
                if t == 0.0 {
                    1.0
                } else {
                    (PI * t).sin() / (PI * t)
                }
            }
            WindowSpec::Gaussian { sigma } => {
                // Sigma <= 0.5 typically
                let t = (n - n_f64 / 2.0) / (sigma * n_f64 / 2.0);
                std::f64::consts::E.powf(-0.5 * t * t)
            }
            WindowSpec::Tukey { alpha } => {
                let t = n / n_f64;
                if t < *alpha / 2.0 {
                    0.5 * (1.0 + (PI * (2.0 * t / alpha - 1.0)).cos())
                } else if t <= 1.0 - *alpha / 2.0 {
                    1.0
                } else {
                    0.5 * (1.0 + (PI * (2.0 * t / alpha - 2.0 / alpha + 1.0)).cos())
                }
            }
            WindowSpec::Kaiser { beta } => {
                let alpha = (PI * beta).abs(); // Some definitions use alpha directly
                let m = n_f64 / 2.0;
                let k = 1.0 - ((n - m) / m).powi(2);
                if k < 0.0 {
                    0.0
                } else {
                    besseli0(alpha * k.sqrt()) / besseli0(alpha)
                }
            }
            WindowSpec::Cauchy { alpha } => {
                let t = (n - n_f64 / 2.0) / (n_f64 / 2.0);
                1.0 / (1.0 + (*alpha * t).powi(2))
            }
            WindowSpec::Poisson { alpha } => {
                let t = (n - n_f64 / 2.0).abs() / (n_f64 / 2.0);
                std::f64::consts::E.powf(-*alpha * t)
            }
            WindowSpec::HannPoisson { alpha } => {
                let t = (n - n_f64 / 2.0).abs() / (n_f64 / 2.0);
                let hann = 0.5 * (1.0 - (2.0 * PI * n / n_f64).cos());
                let poisson = std::f64::consts::E.powf(-*alpha * t);
                hann * poisson
            }
            WindowSpec::Welch => {
                let t = (n - n_f64 / 2.0) / (n_f64 / 2.0);
                1.0 - t * t
            }
            WindowSpec::Parzen => {
                let t = (n - n_f64 / 2.0).abs() / (n_f64 / 2.0);
                if t <= 0.5 {
                    1.0 - 6.0 * t.powi(2) + 6.0 * t.powi(3)
                } else if t <= 1.0 {
                    2.0 * (1.0 - t).powi(3)
                } else {
                    0.0
                }
            }
            WindowSpec::TukeyHarris { alpha } => {
                // Generalized Tukey-Harris interpolation
                let a0 = 0.35875;
                let a1 = 0.48829;
                let a2 = 0.14128;
                let a3 = 0.01168;
                let t = n / n_f64;
                if t < *alpha / 2.0 {
                    a0 - a1 * (2.0 * PI * (t / *alpha)).cos()
                        + a2 * (4.0 * PI * (t / *alpha)).cos()
                        - a3 * (6.0 * PI * (t / *alpha)).cos()
                } else if t <= 1.0 - *alpha / 2.0 {
                    1.0
                } else {
                    let end_t = (1.0 - t) / *alpha;
                    a0 - a1 * (2.0 * PI * end_t).cos()
                        + a2 * (4.0 * PI * end_t).cos()
                        - a3 * (6.0 * PI * end_t).cos()
                }
            }
            WindowSpec::GeneralizedCosine { coeffs } => {
                let mut sum = 0.0;
                for (j, &c) in coeffs.iter().enumerate() {
                    let term = c * (2.0 * PI * (j as f64) * n / n_f64).cos();
                    if j % 2 == 0 {
                        sum += term;
                    } else {
                        sum -= term;
                    }
                }
                sum
            }
            _ => {
                // Fallback to Rectangular for exotic windows (DPSS, DolphChebyshev, etc)
                // DPSS requires eigenvector decomposition; DolphChebyshev requires intense trig evaluation.
                // Left intentionally simple for this foundational iteration.
                1.0
            }
        };

        out[i] = v;
    }

    out
}
