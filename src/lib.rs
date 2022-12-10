pub trait Point<const D: usize>: Copy + std::fmt::Debug {
    fn get_axis(&self, n: usize) -> f32;

    #[inline(always)]
    fn distance_squared(self, b: Self) -> f32 {
        (0..D)
            .into_iter()
            .map(|d| {
                let delta = self.get_axis(d) - b.get_axis(d);
                delta * delta
            })
            .sum::<f32>() as f32
    }
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

impl<'a, const D: usize, P: Point<D>> KdTree<'a, D, P> {
    pub fn from_items(points: &'a [P]) -> Self {
        let mut tree = Vec::with_capacity(points.len());
        let mut point_ids = (0..points.len()).into_iter().collect::<Vec<_>>();

        #[derive(Debug)]
        struct Job {
            start: usize,
            end: usize,
            left_right: usize,
            depth: usize,
            parent: usize,
        }

        let root_job = Job {
            start: 0,
            end: points.len() - 1,
            left_right: 0,
            depth: 0,
            parent: 0,
        };

        let mut is_root = true;
        let mut jobs = vec![root_job];

        while let Some(job) = jobs.pop() {
            let Job {
                start,
                end,
                left_right,
                depth,
                parent,
            } = job;

            let axis = depth % D;
            point_ids[start..end].sort_by(|a, b| {
                points[*a]
                    .get_axis(axis)
                    .partial_cmp(&points[*b].get_axis(axis))
                    .unwrap_or_else(|| std::cmp::Ordering::Equal)
            });
            let pivot_index = (start + end) / 2;

            let tree_index = tree.len();
            tree.push(KdTreeNode {
                parent,
                index: point_ids[pivot_index],
                children: [None, None],
            });

            let new_depth = depth + 1;
            let (left_start, left_end) = (start, pivot_index);
            if left_start != left_end {
                jobs.push(Job {
                    start: left_start,
                    end: left_end,
                    left_right: 0,
                    depth: new_depth,
                    parent: tree_index,
                });
            }

            let (right_start, right_end) = (pivot_index + 1, end);
            if right_start != right_end {
                jobs.push(Job {
                    start: right_start,
                    end: right_end,
                    left_right: 1,
                    depth: new_depth,
                    parent: tree_index,
                });
            }

            if is_root {
                is_root = false;
                continue;
            }

            tree[parent].children[left_right] = Some(tree_index);
        }

        Self { points, tree }
    }

    pub fn nearest_within_buffers(
        &self,
        query_point: P,
        radius: f32,
        result: &mut Vec<usize>,
        to_check: &mut Vec<(usize, usize)>,
    ) {
        let radius_squared = radius * radius;

        let mut querty_point_axis_values = [0.0; D];
        for i in 0..D {
            querty_point_axis_values[i] = query_point.get_axis(i);
        }

        to_check.push((0, 0));
        while let Some((depth, tree_index)) = to_check.pop() {
            let point_index = self.tree[tree_index].index;

            let axis = depth % D;
            let axis_query_point_val = querty_point_axis_values[axis];
            let axis_tree_point_val = self.points[point_index].get_axis(axis);
            let axis_d = axis_tree_point_val - axis_query_point_val;

            let left_first = axis_d >= 0.0;
            let needs_to_go_both = axis_d.abs() <= radius;

            if query_point.distance_squared(self.points[point_index]) <= radius_squared {
                result.push(point_index);
            }

            let first = if left_first { 0 } else { 1 };
            let last = (first + 1) % 2;

            if let Some(child) = self.tree[tree_index].children[first] {
                to_check.push((depth + 1, child));
            }
            if needs_to_go_both {
                if let Some(child) = self.tree[tree_index].children[last] {
                    to_check.push((depth + 1, child));
                }
            }
        }
    }

    pub fn nearest_within(&self, query_point: P, radius: f32) -> Vec<usize> {
        let mut result = vec![];
        let mut to_check = vec![];

        self.nearest_within_buffers(query_point, radius, &mut result, &mut to_check);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arr_5() {
        #[rustfmt::skip]
        let points: [[f32; 2]; 5] = [
            [1.0, 0.0],
            [2.0, 2.0],
            [3.0, -1.0],
            [-1.0, 0.0],
            [0.0, 1.0],
        ];
        let tree = KdTree::from_items(&points);

        dbg!(&tree.tree);

        let nearest = tree.nearest_within([0.0, 0.0], 1.0);
        for point_index in &nearest {
            let point = tree.points[*point_index];
            dbg!(point);
        }
    }

    #[test]
    fn test_arr_8() {
        #[rustfmt::skip]
        let points: [[f32; 2]; 8] = [
            [1.0, 1.0],
            [-3.0, 3.0],
            [-2.0, 0.0],
            [0.0, 1.0],
            [-1.0, -2.0],
            [-3.0, -3.0],
            [3.0, 3.0],
            [2.0, -2.0],
        ];
        let tree = KdTree::from_items(&points);

        dbg!(&tree.tree);

        let nearest = tree.nearest_within([0.0, 0.0], 3.0);
        for point_index in &nearest {
            let point = tree.points[*point_index];
            dbg!(point_index, point);
        }
    }

    #[test]
    fn test_arr_12() {
        let points: [[f32; 3]; 12] = [
            [9.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [11.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
            [6.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [7.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [8.0, 0.0, 0.0],
        ];
        let tree = KdTree::from_items(&points);
        let nearest = tree.nearest_within([0.0, 0.0, 0.0], 2.2);

        for point_index in &nearest {
            let point = tree.points[*point_index];
            dbg!(point);
        }
    }
}

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
