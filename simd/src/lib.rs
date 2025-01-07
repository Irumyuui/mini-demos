use std::arch::x86_64::{
    _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
};

pub fn simd_find_u8(arr: &[u8], target: u8) -> Option<usize> {
    let target_vec = unsafe { _mm256_set1_epi8(target as _) };
    let len = arr.len();
    let mut i = 0;

    unsafe {
        while i <= len - 32 {
            let data_vec = _mm256_loadu_si256(arr.as_ptr().add(i) as _);
            let cmp_ret = _mm256_cmpeq_epi8(data_vec, target_vec);
            let mask = _mm256_movemask_epi8(cmp_ret);

            if mask != 0 {
                for j in 0..32 {
                    if mask & (1 << j) != 0 {
                        return Some(i + j);
                    }
                }
            }
            i += 32;
        }
    }

    for j in i..len {
        if arr[j] == target {
            return Some(j);
        }
    }

    None
}

pub fn binary_find_u8(arr: &[u8], target: u8) -> Option<usize> {
    match arr.binary_search(&target) {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

pub fn linear_find_u8(arr: &[u8], target: u8) -> Option<usize> {
    for (i, e) in arr.iter().enumerate() {
        if *e == target {
            return Some(i);
        }
    }
    None
}
