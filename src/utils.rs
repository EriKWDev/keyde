/// Mostly internal utils like sorting functions and other algorithms
use crate::{Point, SortingStrategy};

pub use heap_sort::*;
pub use shell_sort::*;

/*
    TODO: Decouple sorting from Points trait.
          Could be done using generics:

          fn name<F>( .. )
          where
              F: FnMut(items: &[T], usize, usize) -> std::cmp::Ordering
*/

#[inline(always)]
pub fn point_axis_compare<const D: usize, P>(a: &P, b: &P, axis: usize) -> std::cmp::Ordering
where
    P: Point<D>,
{
    a.get_axis(axis)
        .partial_cmp(&b.get_axis(axis))
        .unwrap_or_else(|| std::cmp::Ordering::Equal)
}

#[inline]
pub fn sort_using_strategy<P, const D: usize>(
    points: &[P],
    indices: &mut [usize],
    axis: usize,
    strategy: &SortingStrategy,
) where
    P: Point<D>,
{
    /*
        TODO: Investigate if other sorting methods are faster.
              Tested and implemented:
                - [X] Merge-sort
                - [X] Quick-sort
                - [X] Shell-sort
                - [X] Heap-sort
    */

    match strategy {
        SortingStrategy::StableSort => {
            indices.sort_by(|a, b| {
                points[*a]
                    .get_axis(axis)
                    .partial_cmp(&points[*b].get_axis(axis))
                    .unwrap_or_else(|| std::cmp::Ordering::Equal)
            });
        }

        SortingStrategy::UnstableSort => {
            indices.sort_unstable_by(|a, b| {
                points[*a]
                    .get_axis(axis)
                    .partial_cmp(&points[*b].get_axis(axis))
                    .unwrap_or_else(|| std::cmp::Ordering::Equal)
            });
        }

        SortingStrategy::ShellSort => {
            crate::utils::shell_sort(points, indices, axis);
        }

        SortingStrategy::HeapSort => {
            crate::utils::heap_sort(points, indices, axis);
        }
    };
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
        loop {
            let left = 2 * root + 1;

            if left > last {
                break;
            }
            let right = left + 1;

            let max = if right <= last
                && points[arr[right]].get_axis(axis) > points[arr[left]].get_axis(axis)
            {
                right
            } else {
                left
            };

            if points[arr[max]].get_axis(axis) > points[arr[root]].get_axis(axis) {
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
