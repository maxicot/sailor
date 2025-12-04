mod heap {
    use core::ptr::addr_of_mut;

    use talc::{Talc, Talck, ClaimOnOom, Span};

    const ARENA_SIZE: usize = 1024 * 1024;
    static mut ARENA: [u8; ARENA_SIZE] = [0; ARENA_SIZE];

    #[global_allocator]
    static TALC: Talck<spin::Mutex<()>, ClaimOnOom> = unsafe {
        Talc::new(ClaimOnOom::new(Span::from_array(addr_of_mut!(ARENA)))).lock()
    };
}
