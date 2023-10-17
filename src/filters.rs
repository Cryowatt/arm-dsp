use crate::cmsis::{q31_t, self as cmsis, q15_t};
use crate::CMSISType;

pub trait BlockFilter<T, const BLOCK_SIZE: u32> {
    fn filter(
        &mut self,
        source: &[T; BLOCK_SIZE as usize],
        destination: &mut [T; BLOCK_SIZE as usize],
    );
}

pub trait BlockDecimateFilter<T, const M: u8, const BLOCK_SIZE: u32>
where
    [T; (BLOCK_SIZE / M as u32) as usize]: Sized,
{
    fn filter(
        &mut self,
        source: &[T; BLOCK_SIZE as usize],
        destination: &mut [T; (BLOCK_SIZE / M as u32) as usize],
    );
}

pub trait BlockInterpolateFilter<T, const L: u8, const BLOCK_SIZE: u32>
where
    [T; (BLOCK_SIZE * L as u32) as usize]: Sized,
{
    fn filter(
        &mut self,
        source: &[T; BLOCK_SIZE as usize],
        destination: &mut [T; (BLOCK_SIZE * L as u32) as usize],
    );
}

pub struct FirFilter<T, const TAPS: u16, const BLOCK_SIZE: u32>
where
    [T; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
{
    state: [T; TAPS as usize + BLOCK_SIZE as usize - 1],
    coefficents: [T; TAPS as usize],
}

pub struct FirDecimateFilter<T, const M: u8, const TAPS: u16, const BLOCK_SIZE: u32>
where
    [T; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
{
    state: [T; TAPS as usize + BLOCK_SIZE as usize - 1],
    coefficents: [T; TAPS as usize],
}

pub struct FirInterpolateFilter<T, const L: u8, const TAPS: u16, const BLOCK_SIZE: u32>
where
    [T; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
{
    state: [T; TAPS as usize + BLOCK_SIZE as usize - 1],
    coefficents: [T; TAPS as usize],
}

impl<T, const TAPS: u16, const BLOCK_SIZE: u32> FirFilter<T, TAPS, BLOCK_SIZE>
where
    T: Default + Copy,
    [T; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
{
    pub fn new(coefficents: [T; TAPS as usize]) -> FirFilter<T, TAPS, BLOCK_SIZE> {
        Self {
            state: [T::default(); { TAPS as usize + BLOCK_SIZE as usize - 1 }],
            coefficents,
        }
    }
}

impl<T, const M: u8, const TAPS: u16, const BLOCK_SIZE: u32>
    FirDecimateFilter<T, M, TAPS, BLOCK_SIZE>
where
    T: Default + Copy,
    [T; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
{
    pub fn new(coefficents: [T; TAPS as usize]) -> FirDecimateFilter<T, M, TAPS, BLOCK_SIZE> {
        if BLOCK_SIZE % M as u32 != 0 {
            panic!("Block size must be a multiple of M");
        }
        Self {
            state: [T::default(); { TAPS as usize + BLOCK_SIZE as usize - 1 }],
            coefficents,
        }
    }
}

macro_rules! FirFilter {
    ($sample_type:ty, $state_type:path, $function:expr) => {
        impl<const TAPS: u16, const BLOCK_SIZE: u32> CMSISType<$state_type>
            for FirFilter<$sample_type, TAPS, BLOCK_SIZE>
        where
            [$sample_type; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
        {
            fn as_cmsis_type(&mut self) -> $state_type {
                $state_type {
                    numTaps: TAPS,
                    pState: self.state.as_mut_ptr(),
                    pCoeffs: self.coefficents.as_ptr(),
                }
            }
        }

        impl<const TAPS: u16, const BLOCK_SIZE: u32> BlockFilter<$sample_type, BLOCK_SIZE>
            for FirFilter<$sample_type, TAPS, BLOCK_SIZE>
        where
            [$sample_type; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
        {
            fn filter(
                &mut self,
                source: &[$sample_type; BLOCK_SIZE as usize],
                destination: &mut [$sample_type; BLOCK_SIZE as usize],
            ) {
                unsafe {
                    $function(
                        &self.as_cmsis_type() as *const $state_type,
                        source.as_ptr(),
                        destination.as_mut_ptr(),
                        BLOCK_SIZE as u32,
                    )
                }
            }
        }
    };
}

macro_rules! FirDecimateFilter {
    ($sample_type:ty, $state_type:path, $function:expr) => {
        impl<const M: u8, const TAPS: u16, const BLOCK_SIZE: u32> CMSISType<$state_type>
            for FirDecimateFilter<$sample_type, M, TAPS, BLOCK_SIZE>
        where
            [$sample_type; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
        {
            fn as_cmsis_type(&mut self) -> $state_type {
                $state_type {
                    M,
                    numTaps: TAPS,
                    pState: self.state.as_mut_ptr(),
                    pCoeffs: self.coefficents.as_ptr(),
                }
            }
        }

        impl<const M: u8, const TAPS: u16, const BLOCK_SIZE: u32>
            BlockDecimateFilter<$sample_type, M, BLOCK_SIZE>
            for FirDecimateFilter<$sample_type, M, TAPS, BLOCK_SIZE>
        where
            [$sample_type; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
            [$sample_type; (BLOCK_SIZE / M as u32) as usize]: Sized,
        {
            fn filter(
                &mut self,
                source: &[$sample_type; BLOCK_SIZE as usize],
                destination: &mut [$sample_type; (BLOCK_SIZE / M as u32) as usize],
            ) {
                unsafe {
                    $function(
                        &self.as_cmsis_type() as *const $state_type,
                        source.as_ptr(),
                        destination.as_mut_ptr(),
                        BLOCK_SIZE as u32,
                    )
                }
            }
        }
    };
}

macro_rules! FirInterpolateFilter {
    ($sample_type:ty, $state_type:path, $function:expr) => {
        impl<const L: u8, const TAPS: u16, const BLOCK_SIZE: u32> CMSISType<$state_type>
            for FirInterpolateFilter<$sample_type, L, TAPS, BLOCK_SIZE>
        where
            [$sample_type; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
        {
            fn as_cmsis_type(&mut self) -> $state_type {
                $state_type {
                    L,
                    phaseLength: TAPS / L as u16,
                    pState: self.state.as_mut_ptr(),
                    pCoeffs: self.coefficents.as_ptr(),
                }
            }
        }

        impl<const L: u8, const TAPS: u16, const BLOCK_SIZE: u32>
            BlockInterpolateFilter<$sample_type, L, BLOCK_SIZE>
            for FirInterpolateFilter<$sample_type, L, TAPS, BLOCK_SIZE>
        where
            [$sample_type; (BLOCK_SIZE * L as u32) as usize]: Sized,
            [$sample_type; TAPS as usize + BLOCK_SIZE as usize - 1]: Sized,
        {
            fn filter(
                &mut self,
                source: &[$sample_type; BLOCK_SIZE as usize],
                destination: &mut [$sample_type; (BLOCK_SIZE*L as u32) as usize],
            ) {
                unsafe {
                    $function(
                        &self.as_cmsis_type() as *const $state_type,
                        source.as_ptr(),
                        destination.as_mut_ptr(),
                        BLOCK_SIZE as u32,
                    )
                }
            }
        }
    };
}

FirFilter!(q15_t, cmsis::arm_fir_instance_q15, cmsis::arm_fir_q15);
FirFilter!(q31_t, cmsis::arm_fir_instance_q31, cmsis::arm_fir_q31);
FirFilter!(f32, cmsis::arm_fir_instance_f32, cmsis::arm_fir_f32);
FirDecimateFilter!(
    q15_t,
    cmsis::arm_fir_decimate_instance_q15,
    cmsis::arm_fir_decimate_q15
);
FirDecimateFilter!(
    q31_t,
    cmsis::arm_fir_decimate_instance_q31,
    cmsis::arm_fir_decimate_q31
);
FirDecimateFilter!(
    f32,
    cmsis::arm_fir_decimate_instance_f32,
    cmsis::arm_fir_decimate_f32
);
FirInterpolateFilter!(
    q15_t,
    cmsis::arm_fir_interpolate_instance_q15,
    cmsis::arm_fir_interpolate_q15
);
FirInterpolateFilter!(
    q31_t,
    cmsis::arm_fir_interpolate_instance_q31,
    cmsis::arm_fir_interpolate_q31
);
FirInterpolateFilter!(
    f32,
    cmsis::arm_fir_interpolate_instance_f32,
    cmsis::arm_fir_interpolate_f32
);
