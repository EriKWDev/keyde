// let pivot = quickselect(points, &mut point_ids[start..end], (end - start) / 2, axis);
// let pivot = start + pivot;
// shell_sort(points, &mut point_ids[start..end], axis);

#[inline]
fn calculate_hash<T: std::hash::Hash>(t: T) -> u64 {
    use std::hash::Hasher;

    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[inline]
fn random_index(len: usize, seed: f32) -> usize {
    let val = calculate_hash((seed * 100.0).round() as usize) as usize;
    val % len
}

pub fn quickselect<P, const D: usize>(
    points: &[P],
    indices: &mut [usize],
    index: usize,
    axis: usize,
) -> usize
where
    P: Point<D>,
{
    let mut pivot_index = random_index(indices.len(), points[index].get_axis(0));
    pivot_index = partition(points, indices, pivot_index, axis);

    match index.cmp(&pivot_index) {
        std::cmp::Ordering::Equal => index,

        std::cmp::Ordering::Less => quickselect(points, &mut indices[0..pivot_index], index, axis),

        std::cmp::Ordering::Greater => quickselect(
            points,
            &mut indices[pivot_index + 1..],
            index - pivot_index - 1,
            axis,
        ),
    }
}

pub fn partition<P, const D: usize>(
    points: &[P],
    indices: &mut [usize],
    pivot_index: usize,
    axis: usize,
) -> usize
where
    P: Point<D>,
{
    let end_index = indices.len() - 1;
    indices.swap(pivot_index, end_index);

    let mut store_index = 0;
    (0..end_index).into_iter().for_each(|i| {
        let a = indices[i];
        let b = indices[end_index];

        let cmp = points[a]
            .get_axis(axis)
            .partial_cmp(&points[b].get_axis(axis))
            .unwrap_or(std::cmp::Ordering::Equal);

        if let std::cmp::Ordering::Less = cmp {
            indices.swap(i, store_index);
            store_index += 1;
        }
    });

    indices.swap(end_index, store_index);
    store_index
}

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
