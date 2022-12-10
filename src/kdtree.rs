use crate::Point;

#[derive(Debug, Clone)]
/// Internal node within the KdTree
pub struct KdTreeNode {
    pub parent: usize,
    pub index: usize,
    pub children: [Option<usize>; 2],
}

#[derive(Debug, Clone)]
/// Depending on the nature of your data, some strategies might work better than others
pub enum KdTreeStrategy {
    UnstableSort,
    StableSort,
    ShellSort,
    HeapSort,
}

impl Default for KdTreeStrategy {
    fn default() -> Self {
        Self::UnstableSort
    }
}

#[derive(Debug, Clone)]
/// A Kd-tree of points with dimension D that uses lifetime semantics to
/// signify that it only works when the provided points have not been modified.
/// Use `KdTreeNoBorrow` to use it without that constraint at your own risk.
pub struct KdTree<'a, const D: usize, P: Point<D>> {
    pub internal: KdTreeNoBorrow<D, P>,
    pub points: &'a [P],
}

impl<'a, const D: usize, P: Point<D>> KdTree<'a, D, P> {
    #[inline(always)]
    pub fn from_points(points: &'a [P]) -> Self {
        Self {
            internal: KdTreeNoBorrow::from_points(points),
            points,
        }
    }

    /// Same as `from_items` but you can pick your own construction/querying strategy
    #[inline(always)]
    pub fn from_points_with_strategy(points: &'a [P], strategy: KdTreeStrategy) -> Self {
        Self {
            internal: KdTreeNoBorrow::from_points_with_strategy(points, strategy),
            points,
        }
    }

    /// Same as `point_indices_within`, but you provide your own buffers. Providing your own buffers
    /// will be more efficient on multiple consecutive queries since you can reuse the allocations made
    /// during the previous queries.
    ///
    /// Indices of points will be inserted into `result` which is not cleared by this function.
    /// `to_check` is assumed to be empty from the start and will be cleared each time after calling this function.
    #[inline(always)]
    pub fn point_indices_within_buffers(
        &self,
        query_point: P,
        radius: f32,
        result: &mut Vec<usize>,
        to_check: &mut Vec<(usize, usize)>,
    ) {
        self.internal.point_indices_within_buffers(
            self.points,
            query_point,
            radius,
            result,
            to_check,
        )
    }

    /// Returns a Vec of indices of the points that are within a hyperssphere of
    /// the specified radius. Note that the distance is determined using Point::distance_squared
    /// which is a euclidian distance by default.
    ///
    /// If you want to allocate your own buffer for multiple consecutive queries, see `point_indices_within_buffers`
    #[inline(always)]
    pub fn point_indices_within(&self, query_point: P, radius: f32) -> Vec<usize> {
        self.internal
            .point_indices_within(self.points, query_point, radius)
    }
}

#[derive(Debug, Clone)]
/// A KdTree of points with dimension D that doesn't use lifetime semantics
pub struct KdTreeNoBorrow<const D: usize, P: Point<D>> {
    pub strategy: KdTreeStrategy,
    pub tree: Vec<KdTreeNode>,
    pub __marker: std::marker::PhantomData<P>,
}

impl<const D: usize, P: Point<D>> KdTreeNoBorrow<D, P> {
    /// See `KdTree`
    pub fn from_points(points: &[P]) -> Self {
        Self::from_points_with_strategy(points, KdTreeStrategy::default())
    }

    /// See `KdTree`
    pub fn from_points_with_strategy(points: &[P], strategy: KdTreeStrategy) -> Self {
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

        /*
            TODO: Pre-sort on all axes before-hand instead of sorting at each level

            TODO: Investigate if other sorting/pivot-picking methods are faster.
                  Tested and implemented:
                    - [X] Merge-sort
                    - [X] Shell-sort
                    - [X] Quick-sort
                    - [ ] Heap-sort
                    - [ ] Median of medians
        */
        while let Some(job) = jobs.pop() {
            let Job {
                start,
                end,
                left_right,
                depth,
                parent,
            } = job;

            let axis = depth % D;
            let pivot_index = match strategy {
                KdTreeStrategy::StableSort => {
                    point_ids[start..end].sort_by(|a, b| {
                        points[*a]
                            .get_axis(axis)
                            .partial_cmp(&points[*b].get_axis(axis))
                            .unwrap_or_else(|| std::cmp::Ordering::Equal)
                    });

                    (start + end) / 2
                }

                KdTreeStrategy::UnstableSort => {
                    point_ids[start..end].sort_unstable_by(|a, b| {
                        points[*a]
                            .get_axis(axis)
                            .partial_cmp(&points[*b].get_axis(axis))
                            .unwrap_or_else(|| std::cmp::Ordering::Equal)
                    });

                    (start + end) / 2
                }

                KdTreeStrategy::ShellSort => {
                    crate::utils::shell_sort(points, &mut point_ids[start..end], axis);

                    (start + end) / 2
                }

                KdTreeStrategy::HeapSort => {
                    crate::utils::heap_sort(points, &mut point_ids[start..end], axis);

                    (start + end) / 2
                }
            };

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

        Self {
            strategy,
            tree,
            __marker: std::marker::PhantomData,
        }
    }

    /// See `KdTree`
    pub fn point_indices_within_buffers(
        &self,
        points: &[P],
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
            let axis_tree_point_val = points[point_index].get_axis(axis);
            let axis_d = axis_tree_point_val - axis_query_point_val;

            let left_first = axis_d >= 0.0;
            let needs_to_go_both = axis_d.abs() <= radius;

            if query_point.distance_squared(points[point_index]) <= radius_squared {
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

    /// See `KdTree`
    pub fn point_indices_within(&self, points: &[P], query_point: P, radius: f32) -> Vec<usize> {
        let mut result = vec![];
        let mut to_check = vec![];

        self.point_indices_within_buffers(points, query_point, radius, &mut result, &mut to_check);

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
        let tree = KdTreeNoBorrow::from_points(&points);

        dbg!(&tree.tree);

        let nearest = tree.point_indices_within(&points, [0.0, 0.0], 1.0);
        for point_index in &nearest {
            let point = points[*point_index];
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
        let tree = KdTree::from_points(&points);

        let nearest = tree.point_indices_within([0.0, 0.0], 3.0);
        for point_index in &nearest {
            let point = tree.points[*point_index];
            dbg!(point_index, point);
        }
    }

    #[test]
    fn test_arr_8_shell() {
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
        let tree = KdTree::from_points_with_strategy(&points, KdTreeStrategy::ShellSort);

        let nearest = tree.point_indices_within([0.0, 0.0], 3.0);
        for point_index in &nearest {
            let point = tree.points[*point_index];
            dbg!(point_index, point);
        }
    }

    #[test]
    fn test_arr_12_non_owning() {
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
        let tree = KdTreeNoBorrow::from_points(&points);
        let nearest = tree.point_indices_within(&points, [0.0, 0.0, 0.0], 2.2);

        for point_index in &nearest {
            let point = points[*point_index];
            dbg!(point);
        }
    }
}
