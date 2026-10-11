//! One arity table for the tuple trait families (unit keeps its NULL semantics).

macro_rules! for_each_tuple {
    ($callback:ident) => {
        $callback!(1, "Tuple1"; A RA 0);
        $callback!(2, "Tuple2"; A RA 0, B RB 1);
        $callback!(3, "Tuple3"; A RA 0, B RB 1, C RC 2);
        $callback!(4, "Tuple4"; A RA 0, B RB 1, C RC 2, D RD 3);
        $callback!(5, "Tuple5"; A RA 0, B RB 1, C RC 2, D RD 3, E RE 4);
        $callback!(6, "Tuple6"; A RA 0, B RB 1, C RC 2, D RD 3, E RE 4, F RF 5);
        $callback!(7, "Tuple7"; A RA 0, B RB 1, C RC 2, D RD 3, E RE 4, F RF 5, G RG 6);
        $callback!(8, "Tuple8"; A RA 0, B RB 1, C RC 2, D RD 3, E RE 4, F RF 5, G RG 6, H RH 7);
        $callback!(9, "Tuple9"; A RA 0, B RB 1, C RC 2, D RD 3, E RE 4, F RF 5, G RG 6, H RH 7, I RI 8);
        $callback!(10, "Tuple10"; A RA 0, B RB 1, C RC 2, D RD 3, E RE 4, F RF 5, G RG 6, H RH 7, I RI 8, J RJ 9);
        $callback!(11, "Tuple11"; A RA 0, B RB 1, C RC 2, D RD 3, E RE 4, F RF 5, G RG 6, H RH 7, I RI 8, J RJ 9, K RK 10);
        $callback!(12, "Tuple12"; A RA 0, B RB 1, C RC 2, D RD 3, E RE 4, F RF 5, G RG 6, H RH 7, I RI 8, J RJ 9, K RK 10, L RL 11);
    };
}

pub(crate) use for_each_tuple;
