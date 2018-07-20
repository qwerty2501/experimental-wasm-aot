
#[inline]
pub const fn bit_width<T>()->usize{
    ::std::mem::size_of::<T>() * 8
}
pub const CPU_BIT_WIDTH:usize = bit_width::<usize>();
pub const PAGE_SIZE:usize = 65536;
pub const DEFAULT_MAXIMUM:usize = 65536;
