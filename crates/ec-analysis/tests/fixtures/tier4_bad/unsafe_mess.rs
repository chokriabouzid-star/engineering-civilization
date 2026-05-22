static mut COUNTER: i32 = 0;

pub fn dangerous(p: *const i32, n: usize) -> i32 {
    let mut r = 0;
    unsafe {
        for i in 0..n {
            r += *p.add(i);
            if r > 100 { if r > 200 { if r > 500 {
                COUNTER += 1;
                r = *p;
            }}}
        }
    }
    r
}
