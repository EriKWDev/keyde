use crate::Point;

macro_rules! impl_point_value {
    ($t: ty) => {
        impl Point<1> for $t {
            #[inline(always)]
            fn get_axis(&self, _n: usize) -> f32 {
                *self as _
            }
        }
    };
}
impl_point_value!(f32);
impl_point_value!(f64);
impl_point_value!(u8);
impl_point_value!(u16);
impl_point_value!(u32);
impl_point_value!(u64);
impl_point_value!(u128);
impl_point_value!(usize);
impl_point_value!(i8);
impl_point_value!(i16);
impl_point_value!(i32);
impl_point_value!(i64);
impl_point_value!(i128);
impl_point_value!(isize);

macro_rules! impl_point_array {
    ($t: ty, $n: literal) => {
        impl Point<$n> for [$t; $n] {
            #[inline(always)]
            fn get_axis(&self, n: usize) -> f32 {
                self[n] as _
            }
        }
    };
}

impl_point_array!(f32, 1);
impl_point_array!(f32, 2);
impl_point_array!(f32, 3);
impl_point_array!(f32, 4);
impl_point_array!(f64, 1);
impl_point_array!(f64, 2);
impl_point_array!(f64, 3);
impl_point_array!(f64, 4);

macro_rules! impl_point_tuple_2 {
    ($t: ty) => {
        impl Point<2> for ($t, $t) {
            #[inline(always)]
            fn get_axis(&self, n: usize) -> f32 {
                match n {
                    0 => self.0 as _,
                    1 => self.1 as _,

                    _ => unreachable!(),
                }
            }
        }
    };
}
impl_point_tuple_2!(f32);
impl_point_tuple_2!(f64);

macro_rules! impl_point_tuple_3 {
    ($t: ty) => {
        impl Point<3> for ($t, $t, $t) {
            #[inline(always)]
            fn get_axis(&self, n: usize) -> f32 {
                match n {
                    0 => self.0 as _,
                    1 => self.1 as _,
                    2 => self.2 as _,

                    _ => unreachable!(),
                }
            }
        }
    };
}
impl_point_tuple_3!(f32);
impl_point_tuple_3!(f64);

macro_rules! impl_point_tuple_4 {
    ($t: ty) => {
        impl Point<4> for ($t, $t, $t, $t) {
            #[inline(always)]
            fn get_axis(&self, n: usize) -> f32 {
                match n {
                    0 => self.0 as _,
                    1 => self.1 as _,
                    2 => self.2 as _,
                    3 => self.3 as _,

                    _ => unreachable!(),
                }
            }
        }
    };
}
impl_point_tuple_4!(f32);
impl_point_tuple_4!(f64);

#[cfg(feature = "glam")]
macro_rules! impl_point_glam {
    ($t: ty, $n: literal) => {
        impl Point<$n> for $t {
            #[inline(always)]
            fn distance_squared(self, b: Self) -> f32 {
                self.distance_squared(b)
            }

            #[inline(always)]
            fn get_axis(&self, n: usize) -> f32 {
                self[n]
            }
        }
    };
}

#[cfg(feature = "glam")]
impl_point_glam!(glam::Vec2, 2);
#[cfg(feature = "glam")]
impl_point_glam!(glam::Vec3A, 3);
#[cfg(feature = "glam")]
impl_point_glam!(glam::Vec3, 3);
#[cfg(feature = "glam")]
impl_point_glam!(glam::Vec4, 4);

#[cfg(test)]
#[cfg(feature = "glam")]
mod glam_tests {
    use super::*;
    use glam::vec3a;

    #[test]
    fn test_vec3a() {
        let points = vec![
            vec3a(7.0, 0.0, 0.0),
            vec3a(2.0, 0.0, 0.0),
            vec3a(3.0, 0.0, 0.0),
            vec3a(1.0, 0.0, 0.0),
            vec3a(4.0, 0.0, 0.0),
            vec3a(8.0, 0.0, 0.0),
            vec3a(5.0, 0.0, 0.0),
            vec3a(0.0, 0.0, 0.0),
            vec3a(6.0, 0.0, 0.0),
        ];
        let tree = KdTree::from_items(&points);

        for point_index in tree.nearest_within(vec3a(0.0, 0.0, 0.0), 2.0) {
            let point = tree.points[point_index];
            dbg!(point);
        }
    }
}
