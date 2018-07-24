macro_rules! error_should_be{
    ($result:expr,$err_ty:ty, $expected_err:pat)=>(match $result {
            Ok(_)=>panic!("should be error"),
            Err(err)=> match err.downcast::<$err_ty>() {
                Ok(err)=> match err {
                    $expected_err => Ok(()),
                    err => Err(err)?,
                },
                Err(err)=> Err(err),
            }
        });
}