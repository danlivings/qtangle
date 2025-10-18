use std::cmp::min;

pub fn chunk<T: Clone>(slice: Vec<T>, chunk_size: Option<usize>) -> Vec<Vec<T>> {
    match chunk_size {
        None => vec![slice],
        Some(chunk_size) => {
            let chunk_count = (slice.len() as f64 / chunk_size as f64).ceil() as usize;

            let mut chunks = Vec::with_capacity(chunk_count);

            for i in 0..chunk_count {
                let from = i * chunk_size;
                let to = min(slice.len(), (i + 1) * chunk_size);
                let s = &slice[from..to];
                chunks.push(Vec::from(s));
            }

            chunks
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_entire_slice_as_one_chunk_when_chunk_size_is_none() {
        let slice = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let result = chunk(slice.clone(), None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], slice);
    }

    #[test]
    fn returns_chunks_of_correct_size_when_chunk_size_is_provided() {
        let slice = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let result = chunk(slice.clone(), Some(3));

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], &slice[0..3]);
        assert_eq!(result[1], &slice[3..6]);
        assert_eq!(result[2], &slice[6..]);
    }

    #[test]
    fn returns_entire_slice_as_one_chunk_when_chunk_size_is_larger_than_input() {
        let slice = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let result = chunk(slice.clone(), Some(200));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], slice);
    }

    #[test]
    fn returns_chunks_of_correct_size_when_slice_is_not_evenly_divisible_by_chunk_size() {
        let slice = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let result = chunk(slice.clone(), Some(4));

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], &slice[0..4]);
        assert_eq!(result[1], &slice[4..8]);
        assert_eq!(result[2], &slice[8..]);
    }
}
