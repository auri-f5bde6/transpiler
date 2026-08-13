#[macro_export]
macro_rules! test_helper {
    ($test_str: expr, [$($x:expr),*]) => {
        {
            let mut lexer= Lexer::from($test_str);
            $(
                assert_eq!(lexer.bump(),Ok($x));
            )*
        }
    };
}
