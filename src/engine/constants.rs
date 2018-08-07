
#[inline]
pub const fn bit_width<T>()->usize{
    ::std::mem::size_of::<T>() * 8
}
pub const CPU_BIT_WIDTH:usize = bit_width::<usize>();