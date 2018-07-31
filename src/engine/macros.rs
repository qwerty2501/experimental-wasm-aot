macro_rules! error_should_be{
    ($result:expr, $expected_err:pat)=>(match $result {
            Ok(_)=>panic!("should be error"),
            Err(err)=> match err.downcast() {
                Ok(err)=> match err {
                    $expected_err => Ok(()),
                    err => Err(err)?,
                },
                Err(err)=> Err(err),
            }
        });
}