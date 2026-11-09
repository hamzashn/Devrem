//! Kısmi pivotlamalı yoğun LU ayrıştırması (gerçek ve karmaşık sayılar).
//!
//! Faz 1–2'deki devre boyutları (yüzlerce düğüm) için yeterlidir. Büyük devreler
//! için ileride seyrek (sparse) bir çözücüyle değiştirilecek; arayüz aynı kalır.

use devrem_core::CircuitError;
use num_complex::Complex64;
use std::ops::{Div, Mul, Sub};

/// LU ayrıştırmasının çalıştığı sayı türü.
pub trait Scalar:
    Copy + Default + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self>
{
    fn magnitude(self) -> f64;
}

impl Scalar for f64 {
    fn magnitude(self) -> f64 {
        self.abs()
    }
}

impl Scalar for Complex64 {
    fn magnitude(self) -> f64 {
        self.norm()
    }
}

#[derive(Debug, Clone)]
pub struct Lu<T: Scalar> {
    n: usize,
    lu: Vec<T>,
    pivots: Vec<usize>,
}

impl<T: Scalar> Lu<T> {
    /// `a` satır öncelikli `n × n` matristir.
    pub fn factor(mut a: Vec<T>, n: usize) -> Result<Self, CircuitError> {
        debug_assert_eq!(a.len(), n * n);
        let mut pivots = vec![0; n];
        for k in 0..n {
            let (p, max) =
                (k..n)
                    .map(|i| (i, a[i * n + k].magnitude()))
                    .fold(
                        (k, -1.0),
                        |best, cur| if cur.1 > best.1 { cur } else { best },
                    );
            if max.is_nan() || max <= 1e-300 || max.is_infinite() {
                return Err(CircuitError::Singular);
            }
            pivots[k] = p;
            if p != k {
                for j in 0..n {
                    a.swap(k * n + j, p * n + j);
                }
            }
            let pivot = a[k * n + k];
            for i in k + 1..n {
                if a[i * n + k].magnitude() == 0.0 {
                    continue;
                }
                let f = a[i * n + k] / pivot;
                a[i * n + k] = f;
                for j in k + 1..n {
                    a[i * n + j] = a[i * n + j] - f * a[k * n + j];
                }
            }
        }
        Ok(Self { n, lu: a, pivots })
    }

    /// `A·x = b` denklemini çözer; sonuç `b` içine yazılır.
    pub fn solve(&self, b: &mut [T]) {
        let n = self.n;
        for (k, &p) in self.pivots.iter().enumerate() {
            b.swap(k, p);
        }
        for i in 0..n {
            let row = &self.lu[i * n..i * n + i];
            let s = row
                .iter()
                .zip(&b[..i])
                .fold(b[i], |acc, (&l, &x)| acc - l * x);
            b[i] = s;
        }
        for i in (0..n).rev() {
            let row = &self.lu[i * n + i + 1..(i + 1) * n];
            let s = row
                .iter()
                .zip(&b[i + 1..])
                .fold(b[i], |acc, (&u, &x)| acc - u * x);
            b[i] = s / self.lu[i * n + i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_with_pivoting() {
        // İlk pivot sıfır: satır değişimi gerekir.
        let a = vec![0.0, 2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 3.0];
        let lu = Lu::factor(a.clone(), 3).unwrap();
        let x_true = [1.0, -2.0, 3.0];
        let mut b: Vec<f64> = (0..3)
            .map(|i| (0..3).map(|j| a[i * 3 + j] * x_true[j]).sum())
            .collect();
        lu.solve(&mut b);
        for (x, t) in b.iter().zip(x_true) {
            assert!((x - t).abs() < 1e-12);
        }
    }

    #[test]
    fn solves_complex() {
        let j = Complex64::new(0.0, 1.0);
        let one = Complex64::new(1.0, 0.0);
        // [[1, j], [j, 1]] · [1, 1] = [1 + j, 1 + j]
        let lu = Lu::factor(vec![one, j, j, one], 2).unwrap();
        let mut b = vec![one + j, one + j];
        lu.solve(&mut b);
        for x in b {
            assert!((x - one).norm() < 1e-12);
        }
    }

    #[test]
    fn detects_singular() {
        let a = vec![1.0, -1.0, -1.0, 1.0];
        assert_eq!(Lu::factor(a, 2).unwrap_err(), CircuitError::Singular);
    }
}
