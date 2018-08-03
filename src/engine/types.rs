
pub trait WasmIntType{}

impl WasmIntType for u32{}
impl WasmIntType for u64{}


pub trait WasmNumberType{}

impl WasmNumberType for i32{}
impl WasmNumberType for i64{}
impl WasmNumberType for f32{}
impl WasmNumberType for f64{}