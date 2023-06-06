pub mod naive {
    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }
}

pub mod gpt {
    use std::arch::x86_64 as arch;

    #[target_feature(enable = "sse2")]
    unsafe fn find_char_in_blocks(
        buf: &[u8],
        ch: u8,
        block_size: usize,
        repeat_size: usize,
    ) -> Result<usize, usize> {
        let ch_repeat = arch::_mm_set1_epi8(ch as i8);

        let mut offset = 0;
        while offset + repeat_size - 1 < buf.len() {
            let read_ptr = buf.as_ptr().add(offset) as *const arch::__m128i;
            let block_ch = arch::_mm_loadu_si128(read_ptr);
            let cmp_res = arch::_mm_cmpeq_epi8(block_ch, ch_repeat);

            let cmp_mask = arch::_mm_movemask_epi8(cmp_res);
            if cmp_mask != 0 {
                return Ok(offset + cmp_mask.trailing_zeros() as usize);
            }

            offset += block_size;
        }

        Err(offset)
    }

    #[target_feature(enable = "avx2")]
    unsafe fn find_char_in_avx2_blocks(
        buf: &[u8],
        ch: u8,
        block_size: usize,
        repeat_size: usize,
    ) -> Result<usize, usize> {
        let ch_repeat = arch::_mm256_set1_epi8(ch as i8);

        let mut offset = 0;
        while offset + repeat_size - 1 < buf.len() {
            let read_ptr = buf.as_ptr().add(offset) as *const arch::__m256i;
            let block_ch = arch::_mm256_loadu_si256(read_ptr);
            let cmp_res = arch::_mm256_cmpeq_epi8(block_ch, ch_repeat);

            let cmp_mask = arch::_mm256_movemask_epi8(cmp_res);
            if cmp_mask != 0 {
                return Ok(offset + cmp_mask.trailing_zeros() as usize);
            }

            offset += block_size;
        }

        Err(offset)
    }

    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        const SSE2_BLOCK_SIZE: usize = 16;
        const AVX2_BLOCK_SIZE: usize = 32;

        const THRESHOLD: usize = 32;

        if haystack.len() < THRESHOLD {
            // Для коротких буферов используем обычный поиск.
            return haystack.iter().position(|&b| b == needle);
        }

        let ch = needle;
        let buf = haystack;

        let result = if is_x86_feature_detected!("avx2") {
            unsafe { find_char_in_avx2_blocks(buf, ch, AVX2_BLOCK_SIZE, 2 * AVX2_BLOCK_SIZE) }
        } else if is_x86_feature_detected!("sse2") {
            unsafe { find_char_in_blocks(buf, ch, SSE2_BLOCK_SIZE, 2 * SSE2_BLOCK_SIZE) }
        } else {
            Err(0)
        };

        match result {
            Ok(pos) => Some(pos),
            Err(offset) => {
                let remaining_buf = &buf[offset..];
                remaining_buf.iter().position(|&c| c == ch).map(|pos| offset + pos)
            }
        }
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
