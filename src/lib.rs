#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![no_std]

mod cmsis;
pub mod filters;

pub trait CMSISType<T> {
    fn as_cmsis_type(&mut self) ->  T;
}