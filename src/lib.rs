pub mod naive {
    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }
}

pub mod gpt {
    use std::arch::x86_64::*;
    use std::mem;

    const SIMD_LANE_SIZE: usize = mem::size_of::<__m128i>();

    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        let needle_simd = unsafe { _mm_set1_epi8(needle as i8) };
        let len = haystack.len();
        let mut i = 0;

        let haystack_ptr = haystack.as_ptr();

        while (len - i) >= SIMD_LANE_SIZE {
            unsafe {
                let chunk = _mm_loadu_si128(haystack_ptr.add(i) as *const _);
                let mask = _mm_cmpeq_epi8(chunk, needle_simd);
                let mask_bits = _mm_movemask_epi8(mask);

                if mask_bits != 0 {
                    let first_nz = mask_bits.trailing_zeros();
                    return Some(i + first_nz as usize);
                }
            }
            i += SIMD_LANE_SIZE;
        }

        // Check the remaining part of the buffer
        for &b in haystack[i..].iter() {
            if b == needle {
                return Some(i);
            }
            i += 1;
        }

        None
    }
}

pub mod libc {
    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        let ptr = unsafe {
            libc::memchr(
                haystack.as_ptr() as *const _,
                needle as libc::c_int,
                haystack.len() as libc::size_t,
            )
        };
        if ptr.is_null() {
            None
        } else {
            Some(ptr as usize - haystack.as_ptr() as usize)
        }
    }
}

pub fn memchr_count(
    needle: u8,
    mut haystack: &[u8],
    memchr: fn(u8, &[u8]) -> Option<usize>,
) -> usize {
    let mut count = 0;
    while let Some(pos) = memchr(needle, haystack) {
        haystack = haystack.split_at(pos + 1).1;
        count += 1;
    }
    count
}
