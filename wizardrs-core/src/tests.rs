use crate::card::color::CardColor::*;
use crate::card::Card;

// Generates a test case for comparing cards
macro_rules! test_cards {
    ($test_number:expr, $a_value:expr, $a_color:expr, $b_value:expr, $b_color:expr, $trump_color:expr, $expected:expr) => {
        paste::paste! {
            #[test]
            fn [<card_compare_ $test_number>] () {
                let a = Card::new($a_value, $a_color).unwrap();
                let b = Card::new($b_value, $b_color).unwrap();

                assert_eq!(a.beats(&b, $trump_color), $expected);
            }
        }
    };
}

test_cards!(1, 1, Blue, 1, Blue, Blue, true);
test_cards!(2, 1, Blue, 2, Blue, Blue, false);
test_cards!(3, 2, Blue, 1, Blue, Blue, true);
test_cards!(4, 0, Blue, 0, Blue, Blue, true);
test_cards!(5, 1, Blue, 0, Blue, Blue, true);
test_cards!(6, 0, Blue, 1, Blue, Blue, false);
test_cards!(7, 14, Blue, 13, Blue, Blue, true);
test_cards!(8, 13, Blue, 14, Blue, Blue, false);
test_cards!(9, 1, Blue, 1, Red, Blue, true);
test_cards!(10, 1, Blue, 1, Red, Red, false);
test_cards!(11, 1, Blue, 1, Red, Green, true);
test_cards!(12, 1, Blue, 2, Red, Green, false);
test_cards!(13, 14, Blue, 1, Red, Red, true);
test_cards!(14, 1, Blue, 14, Red, Blue, false);
test_cards!(15, 1, Blue, 0, Red, Red, true);
test_cards!(16, 0, Blue, 1, Red, Blue, false);
