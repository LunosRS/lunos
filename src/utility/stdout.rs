extern "C" {
    #[link_name = "lunos_fast_stdout_writer"]
    fn lunos_fast_stdout_writer_cpp(buf: *const i8, len: usize) -> usize;
}

pub fn write_stdout(s: &str) {
    unsafe {
        let bytes = s.as_bytes();
        lunos_fast_stdout_writer_cpp(bytes.as_ptr() as *const i8, bytes.len());
    }
}
