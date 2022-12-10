/// Mostly internal utils like sorting functions and other algorithms
use crate::Point;

pub use heap_sort::*;
pub use quicksort::*;
pub use shell_sort::*;

#[derive(Debug, Clone)]
/// Depending on the nature of your data, some strategies might work better than others
pub enum SortingStrategy {
    StableSort,
    UnstableSort,
    ShellSort,
    HeapSort,
    QuickSort,
}

impl Default for SortingStrategy {
    fn default() -> Self {
        Self::QuickSort
    }
}

/*
    TODO: Decouple sorting from Point trait.

          Preferably, all sorting algorithms should be decoupled from the points trait.
          This could be done efficiently by changing all `XX_sort` to instead be
          `XX_sort_by` using generics over a comparison function:

              pub fn XX_sort_by<F>(items: &[T], indices: &mut [usize], cmp: F)
              where
                  F: FnMut(items: &[T], usize, usize, usize) -> std::cmp::Ordering
              { .. }

          All sorting methods using `Points` could then utilize `point_axis_compare`
*/

#[inline]
pub fn sort_using_strategy<P, const D: usize>(
    points: &[P],
    indices: &mut [usize],
    axis: usize,
    strategy: &SortingStrategy,
) where
    P: Point<D>,
{
    match strategy {
        SortingStrategy::StableSort => stable_sort(points, indices, axis),
        SortingStrategy::UnstableSort => unstable_sort(points, indices, axis),
        SortingStrategy::ShellSort => shell_sort(points, indices, axis),
        SortingStrategy::HeapSort => heap_sort(points, indices, axis),
        SortingStrategy::QuickSort => quick_sort(points, indices, axis),
    };
}

#[inline(always)]
pub fn stable_sort<P, const D: usize>(points: &[P], indices: &mut [usize], axis: usize)
where
    P: Point<D>,
{
    indices.sort_by(|a, b| point_axis_compare(points, *a, *b, axis));
}

#[inline(always)]
pub fn unstable_sort<P, const D: usize>(points: &[P], indices: &mut [usize], axis: usize)
where
    P: Point<D>,
{
    indices.sort_unstable_by(|a, b| point_axis_compare(points, *a, *b, axis));
}

#[inline(always)]
pub fn point_axis_compare<const D: usize, P>(
    points: &[P],
    a: usize,
    b: usize,
    axis: usize,
) -> std::cmp::Ordering
where
    P: Point<D>,
{
    points[a]
        .get_axis(axis)
        .partial_cmp(&points[b].get_axis(axis))
        .unwrap_or_else(|| std::cmp::Ordering::Equal)
}

pub mod quicksort {
    use super::*;

    pub fn quick_sort<const D: usize, P>(points: &[P], indices: &mut [usize], axis: usize)
    where
        P: Point<D>,
    {
        let mut stack = Vec::new();
        stack.push((0, indices.len()));

        while let Some((start, end)) = stack.pop() {
            if start >= end {
                continue;
            }

            let pivot = partition(points, indices, start, end, axis);

            stack.push((start, pivot));
            stack.push((pivot + 1, end));
        }
    }

    pub fn partition<const D: usize, P>(
        points: &[P],
        indices: &mut [usize],
        start: usize,
        end: usize,
        axis: usize,
    ) -> usize
    where
        P: Point<D>,
    {
        let mut i = start;
        let pivot = end - 1;
        let pivot_val = points[indices[pivot]].get_axis(axis);

        for j in start..pivot {
            if points[indices[j]].get_axis(axis) < pivot_val {
                indices.swap(i, j);
                i += 1;
            }
        }

        indices.swap(i, pivot);
        i
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_quick_sort() {
            #[rustfmt::skip]
            let points = [1_i32, 7, 56, 34, 576, 2, 4, 5, 6, 7, 9, 10, 9, 1, 2, 3, 100, 23452345, 34, 3, 4545];
            let mut indices = (0..points.len()).into_iter().collect::<Vec<_>>();
            let mut indices_2 = (0..points.len()).into_iter().collect::<Vec<_>>();

            quick_sort(&points, &mut indices, 0);
            indices_2.sort_unstable_by(|a, b| {
                points[*a]
                    .get_axis(0)
                    .partial_cmp(&points[*b].get_axis(0))
                    .unwrap_or_else(|| std::cmp::Ordering::Equal)
            });
            for i in 0..points.len() {
                print!("{}, ", points[indices[i]]);
            }
            println!("");
            for i in 0..points.len() {
                print!("{}, ", points[indices_2[i]]);
            }
            println!("");

            for i in 0..points.len() {
                assert!(points[indices[i]] == points[indices_2[i]]);
            }
        }
    }
}

pub mod shell_sort {
    use super::*;

    pub fn shell_sort<P, const D: usize>(points: &[P], indices: &mut [usize], axis: usize)
    where
        P: Point<D>,
    {
        let len = indices.len();
        let mut gap = len as i32 / 2;

        while gap > 0 {
            for i in gap..len as i32 {
                let temp_i = indices[i as usize];
                let temp = points[temp_i].get_axis(axis);
                let mut j = i;

                while j >= gap && points[indices[j as usize - gap as usize]].get_axis(axis) > temp {
                    indices.swap(j as usize, j as usize - gap as usize);
                    j -= gap;
                }

                indices[j as usize] = temp_i;
            }

            gap /= 2;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_shell_sort() {
            #[rustfmt::skip]
            let points = [1_i32, 7, 56, 34, 576, 2, 4, 5, 6, 7, 9, 10, 9, 1, 2, 3, 100, 23452345, 34, 3, 4545];
            let mut indices = (0..points.len()).into_iter().collect::<Vec<_>>();
            let mut indices_2 = (0..points.len()).into_iter().collect::<Vec<_>>();

            shell_sort(&points, &mut indices, 0);
            indices_2.sort_unstable_by(|a, b| {
                points[*a]
                    .get_axis(0)
                    .partial_cmp(&points[*b].get_axis(0))
                    .unwrap_or_else(|| std::cmp::Ordering::Equal)
            });
            for i in 0..points.len() {
                assert!(points[indices[i]] == points[indices_2[i]]);
            }
        }
    }
}

pub mod heap_sort {
    /// Adapted from https://github.com/TheAlgorithms/Rust/blob/master/src/sorting/heap_sort.rs
    use super::*;

    pub fn heap_sort<P, const D: usize>(points: &[P], indices: &mut [usize], axis: usize)
    where
        P: Point<D>,
    {
        if indices.len() <= 1 {
            return;
        }

        heapify(points, indices, axis);

        (1..indices.len()).rev().for_each(|end| {
            indices.swap(0, end);
            move_down(points, &mut indices[..end], 0, axis);
        });
    }

    #[inline]
    fn heapify<P, const D: usize>(points: &[P], indices: &mut [usize], axis: usize)
    where
        P: Point<D>,
    {
        let last_parent = (indices.len() - 2) / 2;
        (0..=last_parent).rev().for_each(|i| {
            move_down(points, indices, i, axis);
        });
    }

    fn move_down<P, const D: usize>(points: &[P], arr: &mut [usize], mut root: usize, axis: usize)
    where
        P: Point<D>,
    {
        let last = arr.len() - 1;
        let root_value = points[arr[root]].get_axis(axis);

        loop {
            let left = 2 * root + 1;

            if left > last {
                break;
            }

            let right = left + 1;
            let left_value = points[arr[left]].get_axis(axis);

            let (max, max_value) = if right <= last {
                let right_value = points[arr[right]].get_axis(axis);

                if right_value > left_value {
                    (right, right_value)
                } else {
                    (left, left_value)
                }
            } else {
                (left, left_value)
            };

            if max_value > root_value {
                arr.swap(root, max);
            }

            root = max;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_heap_sort() {
            #[rustfmt::skip]
            let points = [1_i32, 7, 56, 34, 576, 2, 4, 5, 6, 7, 9, 10, 9, 1, 2, 3, 100, 23452345, 34, 3, 4545];
            let mut indices = (0..points.len()).into_iter().collect::<Vec<_>>();
            let mut indices_2 = (0..points.len()).into_iter().collect::<Vec<_>>();

            heap_sort(&points, &mut indices, 0);
            indices_2.sort_by(|a, b| {
                points[*a]
                    .get_axis(0)
                    .partial_cmp(&points[*b].get_axis(0))
                    .unwrap_or_else(|| std::cmp::Ordering::Equal)
            });

            for i in 0..points.len() {
                print!("{}, ", points[indices[i]]);
            }
            println!("");
            for i in 0..points.len() {
                print!("{}, ", points[indices_2[i]]);
            }
            println!("");

            for i in 0..points.len() {
                assert!(points[indices[i]] == points[indices_2[i]]);
            }
        }
    }
}
