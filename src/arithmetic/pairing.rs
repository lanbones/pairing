use crate::arithmetic::{CurveAffine, Group as _Group};
use core::ops::Mul;
use group::{
    prime::PrimeCurve, Group, GroupOps, GroupOpsOwned, ScalarMul, ScalarMulOwned,
    UncompressedEncoding,
};

use super::FieldExt;

pub trait Engine: Sized + 'static + Clone {
    /// This is the scalar field of the engine's groups.
    type Scalar: FieldExt;
    type Fr: FieldExt;

    /// The projective representation of an element in G1.
    type G1: PrimeCurve<Scalar = Self::Scalar, Affine = Self::G1Affine>
        + From<Self::G1Affine>
        + GroupOps<Self::G1Affine>
        + GroupOpsOwned<Self::G1Affine>
        + ScalarMul<Self::Scalar>
        + ScalarMulOwned<Self::Scalar>
        + _Group<Scalar = Self::Scalar>;

    /// The affine representation of an element in G1.
    type G1Affine: PairingCurveAffine<
            ScalarExt = Self::Scalar,
            CurveExt = Self::G1,
            Pair = Self::G2Affine,
            PairingResult = Self::Gt,
        > + From<Self::G1>
        + Mul<Self::Scalar, Output = Self::G1>
        + for<'a> Mul<&'a Self::Scalar, Output = Self::G1>;

    /// The projective representation of an element in G2.
    type G2: PrimeCurve<Scalar = Self::Scalar, Affine = Self::G2Affine>
        + From<Self::G2Affine>
        + GroupOps<Self::G2Affine>
        + GroupOpsOwned<Self::G2Affine>
        + ScalarMul<Self::Scalar>
        + ScalarMulOwned<Self::Scalar>;

    /// The affine representation of an element in G2.
    type G2Affine: PairingCurveAffine<
            ScalarExt = Self::Scalar,
            CurveExt = Self::G2,
            Pair = Self::G1Affine,
            PairingResult = Self::Gt,
        > + From<Self::G2>
        + Mul<Self::Scalar, Output = Self::G2>
        + for<'a> Mul<&'a Self::Scalar, Output = Self::G2>;

    /// The extension field that hosts the target group of the pairing.
    type Gt: Group<Scalar = Self::Scalar>
        + ScalarMul<Self::Scalar>
        + ScalarMulOwned<Self::Scalar>
        + ff::Field
        + MillerLoopResult<Gt = Self::Gt>;

    /// Invoke the pairing function `G1 x G2 -> Gt` without the use of precomputation and
    /// other optimizations.
    fn pairing(p: &Self::G1Affine, q: &Self::G2Affine) -> Self::Gt;
}

/// Affine representation of an elliptic curve point that can be used
/// to perform pairings.
pub trait PairingCurveAffine: CurveAffine + UncompressedEncoding {
    type Pair: PairingCurveAffine<Pair = Self>;
    type PairingResult: Group;

    /// Perform a pairing
    fn pairing_with(&self, other: &Self::Pair) -> Self::PairingResult;
}

/// An engine that can compute sums of pairings in an efficient way.
pub trait MultiMillerLoop: Engine {
    /// The prepared form of `Self::G2Affine`.
    type G2Prepared: Clone + Send + Sync + From<Self::G2Affine>;

    /// Computes $$\sum_{i=1}^n \textbf{ML}(a_i, b_i)$$ given a series of terms
    /// $$(a_1, b_1), (a_2, b_2), ..., (a_n, b_n).$$
    fn multi_miller_loop(terms: &[(&Self::G1Affine, &Self::G2Prepared)]) -> Self::Gt;
}

//refer "On Proving Pairings"(https://eprint.iacr.org/2024/640.pdf)
pub trait MultiMillerLoopOnProvePairing: MultiMillerLoop {
    type G2OnProvePrepared: Clone + Send + Sync + From<Self::G2Affine>;
    //if support this scheme
    fn support_on_prove_pairing() -> bool;

    // verify miller loop result on project coordinates
    fn multi_miller_loop_c_wi(
        c: &Self::Gt,
        wi: &Self::Gt,
        terms: &[(&Self::G1Affine, &Self::G2Prepared)],
    ) -> Self::Gt;

    // to calc miller loop on affine coordinates
    fn multi_miller_loop_on_prove_pairing_prepare(
        terms: &[(&Self::G1Affine, &Self::G2OnProvePrepared)],
    ) -> Self::Gt;

    // to verify miller loop of affine coordinates with c&wi
    fn multi_miller_loop_on_prove_pairing(
        c: &Self::Gt,
        wi: &Self::Gt,
        terms: &[(&Self::G1Affine, &Self::G2OnProvePrepared)],
    ) -> Self::Gt;

    fn get_g2_on_prove_prepared_coeffs(
        p: &Self::G2OnProvePrepared,
    ) -> Vec<(
        (
            <Self::G1Affine as CurveAffine>::Base,
            <Self::G1Affine as CurveAffine>::Base,
        ),
        (
            <Self::G1Affine as CurveAffine>::Base,
            <Self::G1Affine as CurveAffine>::Base,
        ),
    )>;

    fn get_g2_on_prove_prepared_init_q(p: &Self::G2OnProvePrepared) -> Self::G2Affine;
}

/// Represents results of a Miller loop, one of the most expensive portions of the pairing
/// function.
///
/// `MillerLoopResult`s cannot be compared with each other until
/// [`MillerLoopResult::final_exponentiation`] is called, which is also expensive.
pub trait MillerLoopResult {
    /// The extension field that hosts the target group of the pairing.
    type Gt: Group;

    /// This performs a "final exponentiation" routine to convert the result of a Miller
    /// loop into an element of [`MillerLoopResult::Gt`], so that it can be compared with
    /// other elements of `Gt`.
    fn final_exponentiation(&self) -> Self::Gt;
}
