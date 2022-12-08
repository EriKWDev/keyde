pub trait Point<const D: usize>: Copy + std::fmt::Debug {
    fn distance_squared(self, b: Self) -> f32;
    fn get_axis(&self, n: usize) -> f32;
}

#[derive(Debug)]
pub struct KdTreeNode {
    pub parent: usize,
    pub index: usize,
    pub children: [Option<usize>; 2],
}

#[derive(Debug)]
pub struct KdTree<'a, const D: usize, P: Point<D>> {
    pub points: &'a [P],
    pub tree: Vec<KdTreeNode>,
}

pub fn tree_build<'a, const D: usize, P>(
    points: &'a [P],
    buf: &mut Vec<KdTreeNode>,
    parent: usize,
    left_right: usize,
    ids: &mut [usize],
    depth: usize,
) where
    P: Point<D>,
{
    if ids.is_empty() {
        return;
    }

    let axis = depth % D;
    ids.sort_by(|a, b| {
        points[*a]
            .get_axis(axis)
            .partial_cmp(&points[*b].get_axis(axis))
            .unwrap_or_else(|| std::cmp::Ordering::Equal)
    });
    let median = ids.len() / 2;
    let index = buf.len();

    buf.push(KdTreeNode {
        parent,
        index: ids[median],
        children: [None, None],
    });
    buf[parent].children[left_right] = Some(index);

    let left = &mut ids[..median];
    tree_build(points, buf, index, 0, left, depth + 1);
    let right = &mut ids[median + 1..];
    tree_build(points, buf, index, 1, right, depth + 1);
}

impl<'a, const D: usize, P: Point<D>> KdTree<'a, D, P> {
    pub fn from_items(points: &'a [P]) -> Self {
        let mut tree = Vec::with_capacity(points.len());
        let mut ids = (0..points.len()).into_iter().collect::<Vec<_>>();

        ids.sort_by(|a, b| {
            points[*a]
                .get_axis(0)
                .partial_cmp(&points[*b].get_axis(0))
                .unwrap_or_else(|| std::cmp::Ordering::Equal)
        });
        let median = ids.len() / 2;

        tree.push(KdTreeNode {
            parent: 0, // NOTE: dummy
            index: ids[median],
            children: [None, None],
        });

        let left = &mut ids[..median];
        tree_build(points, &mut tree, 0, 0, left, 1);
        let right = &mut ids[median + 1..];
        tree_build(points, &mut tree, 0, 1, right, 1);

        Self { points, tree }
    }

    pub fn nearest_within(&self, point: P, radius: f32) -> Vec<usize> {
        let radius_squared = radius * radius;

        let mut result = vec![];
        let mut to_check = vec![(0, 0)];
        let mut has_had_it = false;

        while let Some((depth, current)) = to_check.pop() {
            let index = self.tree[current].index;

            if self.points[index].distance_squared(point) <= radius_squared {
                result.push(index);
                has_had_it = true;
            } else if has_had_it {
                return result;
            }

            let axis = depth % D;
            let axis_d = point.get_axis(axis) - self.points[index].get_axis(axis);

            let visit_left = axis_d >= 0.0;

            if visit_left {
                if let Some(left) = self.tree[current].children[0] {
                    to_check.push((depth + 1, left));
                }
                if let Some(right) = self.tree[current].children[1] {
                    to_check.push((depth + 1, right));
                }
            } else {
                if let Some(right) = self.tree[current].children[1] {
                    to_check.push((depth + 1, right));
                }
                if let Some(left) = self.tree[current].children[0] {
                    to_check.push((depth + 1, left));
                }
            }
        }

        result
    }
}

macro_rules! impl_point_array {
    ($t: ty, $n: literal) => {
        impl Point<$n> for [$t; $n] {
            #[inline(always)]
            fn distance_squared(self, b: Self) -> f32 {
                self.iter()
                    .zip(b.iter())
                    .map(|(x, y)| ((*x) - (*y)) * ((*x) - (*y)))
                    .sum::<$t>() as _
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arr() {
        let points = [
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
            [6.0, 0.0, 0.0],
            [7.0, 0.0, 0.0],
            [8.0, 0.0, 0.0],
            [9.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [11.0, 0.0, 0.0],
        ];
        let tree = KdTree::from_items(&points);
        let nearest = tree.nearest_within([0.0, 0.0, 0.0], 2.0);

        for point_index in &nearest {
            let point = tree.points[*point_index];
            dbg!(point);
            assert!(*point_index < 3);
        }

        assert!(nearest.len() == 3);
    }
}

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
            vec3a(0.0, 0.0, 0.0),
            vec3a(1.0, 0.0, 0.0),
            vec3a(2.0, 0.0, 0.0),
            vec3a(3.0, 0.0, 0.0),
            vec3a(4.0, 0.0, 0.0),
            vec3a(5.0, 0.0, 0.0),
            vec3a(6.0, 0.0, 0.0),
            vec3a(7.0, 0.0, 0.0),
            vec3a(8.0, 0.0, 0.0),
        ];
        let tree = KdTree::from_items(&points);

        for point_index in tree.nearest_within(vec3a(0.0, 0.0, 0.0), 2.0) {
            let point = tree.points[point_index];
            dbg!(point);
            assert!(point_index < 3);
        }
    }
}
