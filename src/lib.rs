pub mod naive {
    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
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
