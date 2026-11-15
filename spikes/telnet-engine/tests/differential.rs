//! This engine against libmudtelnet, on streams where both must agree.

use proptest::prelude::*;
use telnet_engine::oracle::{Token, encode, expected, ours, reference, split};
use telnet_engine::{EOR, GA, NOP, opt};

fn plain_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(0u8..255, 1..40)
}

fn token() -> impl Strategy<Value = Token> {
    prop_oneof![
        6 => plain_bytes().prop_map(Token::Data),
        2 => prop::sample::select(vec![GA, EOR, NOP]).prop_map(Token::Prompt),
        2 => (
            prop::sample::select(vec![opt::GMCP, opt::MSSP, opt::TTYPE]),
            prop::collection::vec(0u8..255, 0..40),
        )
            .prop_map(|(o, p)| Token::Subnegotiation(o, p)),
        1 => Just(Token::StartCompression),
        1 => Just(Token::EndCompression),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn agrees_with_libmudtelnet(
        tokens in prop::collection::vec(token(), 0..30),
        cuts in prop::collection::vec(any::<usize>(), 0..12),
    ) {
        let bytes = encode(&tokens);
        let reads = split(&bytes, &cuts);
        let ours = ours(&reads);
        prop_assert_eq!(&ours, &expected(&tokens));
        prop_assert_eq!(&ours, &reference(&reads));
    }
}
