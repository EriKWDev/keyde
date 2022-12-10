//! The implementation of a spacial query structure knonw as a `Kd-tree`
use crate::{Point, SortingStrategy};

#[derive(Debug, Clone)]
/// Internal node within the KdTree
pub struct KdTreeNode {
    pub parent: usize,
    pub index: usize,
    pub children: [Option<usize>; 2],
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
    /// Constructs a new KdTree using the points provided and defualt settings
    #[inline(always)]
    pub fn from_points(points: &'a [P]) -> Self {
        Self {
            internal: KdTreeNoBorrow::from_points(points),
            points,
        }
    }

    /// Same as `from_points` but you can pick your own construction/querying strategy
    #[inline(always)]
    pub fn from_points_with_strategy(points: &'a [P], strategy: &SortingStrategy) -> Self {
        Self {
            internal: KdTreeNoBorrow::from_points_with_strategy(points, strategy),
            points,
        }
    }

    /// Same as `from_points_with_strategy` but uses the pre-sort optimization
    #[inline(always)]
    pub fn from_points_presort_with_strategy(points: &'a [P], strategy: &SortingStrategy) -> Self {
        Self {
            internal: KdTreeNoBorrow::from_points_presort_with_strategy(points, strategy),
            points,
        }
    }

    /// Allows you to specify your own point sorter function. See `from_points_with_strategy`
    /// if you instead want to chose from some pre-provided algorithms.
    ///
    /// Usually not needed, but for full flexibility is provided anyway.
    pub fn from_points_with_points_sorter<F>(points: &'a [P], points_sorter: F) -> Self
    where
        F: FnMut(&[P], &mut [usize], usize),
    {
        Self {
            internal: KdTreeNoBorrow::from_points_with_points_sorter(points, points_sorter),
            points,
        }
    }

    /// Same as `from_points_with_points_sorter`, but uses the pre-sort optimization
    pub fn from_points_presort_with_points_sorter<F>(points: &'a [P], points_sorter: F) -> Self
    where
        F: FnMut(&[P], &mut [usize], usize),
    {
        Self {
            internal: KdTreeNoBorrow::from_points_presort_with_points_sorter(points, points_sorter),
            points,
        }
    }

    /// Same as `point_indices_within`, but you provide your own buffers. Providing your own buffers
    /// will be more efficient on multiple consecutive queries since you can reuse the allocations made
    /// during the previous queries.
    ///
    /// Indices of points will be inserted into `result` which is not cleared by this function.
    /// `stack` is assumed to be empty from the start and will be cleared each time after calling this function.
    #[inline(always)]
    pub fn point_indices_within_buffers(
        &self,
        query_point: P,
        radius: f32,
        result: &mut Vec<usize>,
        stack: &mut Vec<(usize, usize)>,
    ) {
        self.internal
            .point_indices_within_buffers(self.points, query_point, radius, result, stack)
    }

    /// Returns a Vec of indices of the points that are within a hyperssphere of
    /// the specified radius. Note that the distance is determined using `Point::distance_squared`
    /// which is a euclidian distance by default.
    ///
    /// If you want to allocate your own buffer for multiple consecutive queries, see `point_indices_within_buffers`
    #[inline(always)]
    pub fn point_indices_within(&self, query_point: P, radius: f32) -> Vec<usize> {
        self.internal
            .point_indices_within(self.points, query_point, radius)
    }

    #[inline(always)]
    pub fn iter_point_indices_within_buffers(
        &self,
        query_point: P,
        radius: f32,
        stack: &'a mut Vec<(usize, usize)>,
    ) -> IndicesWithinIterator<'_, D, P> {
        self.internal
            .iter_point_indices_within_buffers(self.points, query_point, radius, stack)
    }
}

#[derive(Debug, Clone)]
/// A KdTree of points with dimension D that doesn't use lifetime semantics
pub struct KdTreeNoBorrow<const D: usize, P: Point<D>> {
    pub tree: Vec<KdTreeNode>,
    pub __marker: std::marker::PhantomData<P>,
}

impl<const D: usize, P: Point<D>> KdTreeNoBorrow<D, P> {
    /// See `KdTree`
    pub fn from_points(points: &[P]) -> Self {
        /*
            TODO: Switch to using presort by default once it is implemented
        */
        Self::from_points_with_strategy(points, &SortingStrategy::default())
    }

    /// See `KdTree`
    pub fn from_points_with_strategy(points: &[P], strategy: &SortingStrategy) -> Self {
        let points_sorter = match strategy {
            SortingStrategy::StableSort => crate::utils::stable_sort,
            SortingStrategy::UnstableSort => crate::utils::unstable_sort,
            SortingStrategy::ShellSort => crate::utils::shell_sort,
            SortingStrategy::QuickSort => crate::utils::quick_sort,
            SortingStrategy::HeapSort => crate::utils::heap_sort,
        };

        Self::from_points_with_points_sorter(points, points_sorter)
    }

    /// See `KdTree`
    pub fn from_points_presort_with_strategy(points: &[P], strategy: &SortingStrategy) -> Self {
        let points_sorter = match strategy {
            SortingStrategy::StableSort => crate::utils::stable_sort,
            SortingStrategy::UnstableSort => crate::utils::unstable_sort,
            SortingStrategy::ShellSort => crate::utils::shell_sort,
            SortingStrategy::QuickSort => crate::utils::quick_sort,
            SortingStrategy::HeapSort => crate::utils::heap_sort,
        };

        Self::from_points_presort_with_points_sorter(points, points_sorter)
    }

    /// See `KdTree`
    pub fn from_points_with_points_sorter<F>(points: &[P], mut points_sorter: F) -> Self
    where
        F: FnMut(&[P], &mut [usize], usize),
    {
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
            let pivot_index = (start + end) / 2;

            points_sorter(points, &mut point_ids[start..end], axis);

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

            if depth > 0 {
                /*
                    NOTE: Root has no parent so this only happens when we are
                          not root
                */

                tree[parent].children[left_right] = Some(tree_index);
            }
        }

        Self {
            tree,
            __marker: std::marker::PhantomData,
        }
    }

    /// See `KdTree`
    pub fn from_points_presort_with_points_sorter<F>(points: &[P], mut points_sorter: F) -> Self
    where
        F: FnMut(&[P], &mut [usize], usize),
    {
        let mut tree = Vec::with_capacity(points.len());

        let n = points.len();
        let mut sorted_axis_ids = (0..D)
            .map(|axis| {
                let mut ids = (0..n).collect::<Vec<_>>();
                points_sorter(points, &mut ids, axis);
                ids
            })
            .collect::<Vec<_>>();

        let mut point_id_to_sorted_axis_index = (0..D).map(|axis| {
            let mut map = vec![0; n];

            sorted_axis_ids[axis]
                .iter()
                .enumerate()
                .for_each(|(i, value)| {
                    map[*value] = i;
                });

            map
        });

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
            let pivot_index = (start + end) / 2;
            let relevant_ids = &sorted_axis_ids[axis][start..end];

            let tree_index = tree.len();
            tree.push(KdTreeNode {
                parent,
                index: relevant_ids[pivot_index],
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

            if depth > 0 {
                /*
                    NOTE: Root has no parent so this only happens when we are
                          not root
                */

                tree[parent].children[left_right] = Some(tree_index);
            }
        }

        Self {
            tree,
            __marker: std::marker::PhantomData,
        }
    }

    /// See `KdTree`
    pub fn iter_point_indices_within_buffers<'a>(
        &'a self,
        points: &'a [P],
        query_point: P,
        radius: f32,
        stack: &'a mut Vec<(usize, usize)>,
    ) -> IndicesWithinIterator<'_, D, P> {
        let radius_squared = radius * radius;

        let mut query_point_axis_values = [0.0; D];
        for i in 0..D {
            query_point_axis_values[i] = query_point.get_axis(i);
        }

        stack.push((0, 0));

        IndicesWithinIterator {
            stack,
            tree: self,
            points,
            radius_squared,
            radius,
            query_point_axis_values,
            query_point,
        }
    }

    /// See `KdTree`
    #[inline(always)]
    pub fn point_indices_within_buffers(
        &self,
        points: &[P],
        query_point: P,
        radius: f32,
        result: &mut Vec<usize>,
        stack: &mut Vec<(usize, usize)>,
    ) {
        let radius_squared = radius * radius;

        let mut querty_point_axis_values = [0.0; D];
        for i in 0..D {
            querty_point_axis_values[i] = query_point.get_axis(i);
        }

        stack.push((0, 0));
        while let Some((depth, tree_index)) = stack.pop() {
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
                stack.push((depth + 1, child));
            }
            if needs_to_go_both {
                if let Some(child) = self.tree[tree_index].children[last] {
                    stack.push((depth + 1, child));
                }
            }
        }
    }

    /// See `KdTree`
    #[inline(always)]
    pub fn point_indices_within(&self, points: &[P], query_point: P, radius: f32) -> Vec<usize> {
        let mut result = vec![];
        let mut stack = vec![];

        self.point_indices_within_buffers(points, query_point, radius, &mut result, &mut stack);

        result
    }
}

/// Iterator over indices of points in a KdTree within a hypersphere of `radius` using the
/// euclidean distance function `Point::distance_squared`
pub struct IndicesWithinIterator<'a, const D: usize, P: Point<D>> {
    pub stack: &'a mut Vec<(usize, usize)>,
    pub tree: &'a KdTreeNoBorrow<D, P>,
    pub points: &'a [P],
    pub radius_squared: f32,
    pub radius: f32,
    pub query_point_axis_values: [f32; D],
    pub query_point: P,
}

impl<'a, const D: usize, P: Point<D>> std::iter::Iterator for IndicesWithinIterator<'a, D, P> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((depth, tree_index)) = self.stack.pop() {
            let point_index = self.tree.tree[tree_index].index;

            let axis = depth % D;
            let axis_query_point_val = self.query_point_axis_values[axis];
            let axis_tree_point_val = self.points[point_index].get_axis(axis);
            let axis_d = axis_tree_point_val - axis_query_point_val;

            let left_first = axis_d >= 0.0;
            let needs_to_go_both = axis_d.abs() <= self.radius;

            let first = if left_first { 0 } else { 1 };
            let last = (first + 1) % 2;

            if let Some(child) = self.tree.tree[tree_index].children[first] {
                self.stack.push((depth + 1, child));
            }
            if needs_to_go_both {
                if let Some(child) = self.tree.tree[tree_index].children[last] {
                    self.stack.push((depth + 1, child));
                }
            }

            if self.query_point.distance_squared(self.points[point_index]) <= self.radius_squared {
                return Some(point_index);
            }
        }

        None
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
        let tree = KdTree::from_points_with_strategy(&points, &SortingStrategy::ShellSort);

        let nearest = tree.point_indices_within([0.0, 0.0], 3.0);
        for point_index in &nearest {
            let point = tree.points[*point_index];
            dbg!(point_index, point);
        }
    }

    #[test]
    fn test_arr_8_quick_iter() {
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
        let tree = KdTree::from_points_with_strategy(&points, &SortingStrategy::QuickSort);

        let mut buffer = vec![];
        let nearest = tree.iter_point_indices_within_buffers([0.0, 0.0], 3.0, &mut buffer);
        for point_index in nearest {
            let point = tree.points[point_index];
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
