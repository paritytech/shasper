use ssz::{Encode, Decode, FixedVec};
use core::marker::PhantomData;
use core::fmt::Debug;
use typenum::*;

fn t<T: Debug + Encode + Decode + PartialEq>(value: T, expected: &[u8]) {
	let encoded = value.encode();
	assert_eq!(&encoded[..], expected);
	let decoded = T::decode(&mut &encoded[..]).unwrap();
	assert_eq!(value, decoded);
}

#[test]
fn spec() {
	t(false, &[0x00]); // boolean F
	t(true, &[0x01]); // boolean T
    t(0u8, &[0x00]); // uint8 00
    t(1u8, &[0x01]); // uint8 01
    t(0xabu8, &[0xab]); // uint8 ab
    t(0x0000u16, &[0x00, 0x00]); // uint16 0000
    t(0xabcdu16, &[0xcd, 0xab]); // uint16 abcd
    t(0x00000000u32, &[0x00, 0x00, 0x00, 0x00]); // uint32 00000000
    t(0x01234567u32, &[0x67, 0x45, 0x23, 0x01]); // uint32 01234567

    // uint64 0000000000000000
    t(0x0000000000000000u64, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    // uint64 0123456789abcdef
    t(0x0123456789abcdefu64, &[0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01]);

    // bitvector TTFTFTFF
    t(FixedVec::<bool, U8>(vec![true, true, false, true, false, true, false, false], PhantomData), &[0x2b]);
    // bitvector FTFT
    t(FixedVec::<bool, U4>(vec![false, true, false, true], PhantomData), &[0x0a]);
    // bitvector FTF
    t(FixedVec::<bool, U3>(vec![false, true, false], PhantomData), &[0x02]);
    // bitvector TFTFFFTTFT
    t(FixedVec::<bool, U10>(vec![true, false, true, false, false, false, true, true, false, true], PhantomData), &[0xc5, 0x02]);
    // bitvector TFTFFFTTFTFFFFTT
    t(FixedVec::<bool, U16>(vec![true, false, true, false, false, false, true, true, false, true,
                                 false, false, false, false, true, true], PhantomData),
      &[0xc5, 0xc2]);
    // long bitvector
    {
        let mut v = Vec::new();
        for _ in 0..512 {
            v.push(true);
        }
        t(FixedVec::<bool, U512>(v, PhantomData),
          &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    }
}