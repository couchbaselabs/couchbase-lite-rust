extern crate couchbase_lite;

//
// use couchbase_lite::slice::{bytes_as_slice, Slice};
// use couchbase_lite::c_api::FLSlice;
// use std::ffi::c_void;
//
// /*#[test]
// fn slice_drop_data() {
//     let slice = {
//         let vec = vec![42];
//         bytes_as_slice(&vec)
//     };
//     assert_eq!(slice.to_string(), Some("*".to_string()));
// }*/
//
// #[test]
// fn slice_drop_u8() {
//     fn lost_borrow(_vec: Vec<u8>) {}
//
//     let vec = vec![42];
//     let slice = bytes_as_slice(&vec);
//     lost_borrow(vec);
//     assert_eq!(slice.to_string(), Some("abc".to_string()));
// }
