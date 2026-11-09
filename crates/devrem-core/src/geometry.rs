//! Izgara koordinatları.
//!
//! Tüm konumlar ızgara birimindedir (tam sayı). Ekran koordinat sistemi gibi
//! `y` aşağı doğru artar.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

/// Izgara üzerinde bir nokta. JSON'da `[x, y]` olarak saklanır.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Point(pub i32, pub i32);

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self(x, y)
    }

    /// Noktayı orijin etrafında `rot × 90°` saat yönünde döndürür.
    #[must_use]
    pub const fn rotate(self, rot: u8) -> Self {
        match rot % 4 {
            0 => self,
            1 => Self(-self.1, self.0),
            2 => Self(-self.0, -self.1),
            _ => Self(self.1, -self.0),
        }
    }

    fn dist2(self, other: Self) -> i64 {
        let dx = i64::from(self.0 - other.0);
        let dy = i64::from(self.1 - other.1);
        dx * dx + dy * dy
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

/// `p` noktası `a`–`b` doğru parçasının üzerinde ve uç noktalardan farklı mı?
pub fn strictly_inside_segment(p: Point, a: Point, b: Point) -> bool {
    if p == a || p == b {
        return false;
    }
    let cross =
        i64::from(b.0 - a.0) * i64::from(p.1 - a.1) - i64::from(b.1 - a.1) * i64::from(p.0 - a.0);
    cross == 0
        && p.0 >= a.0.min(b.0)
        && p.0 <= a.0.max(b.0)
        && p.1 >= a.1.min(b.1)
        && p.1 <= a.1.max(b.1)
}

/// Noktaları `a`'ya olan uzaklığa göre sıralar.
pub(crate) fn sort_along(points: &mut [Point], a: Point) {
    points.sort_by_key(|p| p.dist2(a));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_is_clockwise_on_screen() {
        assert_eq!(Point(1, 0).rotate(1), Point(0, 1));
        assert_eq!(Point(1, 0).rotate(2), Point(-1, 0));
        assert_eq!(Point(1, 0).rotate(3), Point(0, -1));
        assert_eq!(Point(2, 3).rotate(4), Point(2, 3));
    }

    #[test]
    fn inside_segment() {
        let (a, b) = (Point(0, 0), Point(4, 0));
        assert!(strictly_inside_segment(Point(2, 0), a, b));
        assert!(!strictly_inside_segment(Point(0, 0), a, b));
        assert!(!strictly_inside_segment(Point(5, 0), a, b));
        assert!(!strictly_inside_segment(Point(2, 1), a, b));
        assert!(strictly_inside_segment(
            Point(1, 1),
            Point(0, 0),
            Point(3, 3)
        ));
    }
}
