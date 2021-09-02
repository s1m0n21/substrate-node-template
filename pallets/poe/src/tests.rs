use crate::mock::*;
use super::*;
use frame_support::{assert_ok, assert_noop};

#[test]
fn create_proof_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_proof(Origin::signed(1), claim.clone()));
        assert_eq!(
            Proofs::<Test>::get(&claim),
            Some((1, frame_system::Pallet::<Test>::block_number()))
        );
    })
}

#[test]
fn create_proof_failed_when_proof_already_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_proof(Origin::signed(1), claim.clone()));
        assert_noop!(PoeModule::create_proof(
            Origin::signed(1), claim.clone()),
            Error::<Test>::ProofAlreadyExist
        );
    })
}

#[test]
fn create_proof_failed_when_proof_exceeds_length_limit() {
    new_test_ext().execute_with(|| {
        let claim: Vec<u8> = vec![0; 256];

        assert_noop!(
            PoeModule::create_proof(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofExceedsLengthLimit
        );
    })
}

#[test]
fn revoke_proof_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_proof(Origin::signed(1), claim.clone()));
        assert_ok!(PoeModule::revoke_proof(Origin::signed(1), claim.clone()));
        assert_eq!(Proofs::<Test>::get(&claim), None);
    })
}

#[test]
fn revoke_proof_failed_when_proof_is_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_noop!(
            PoeModule::revoke_proof(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofNotExist
        );
    })
}

#[test]
fn revoke_proof_failed_when_caller_is_not_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_proof(Origin::signed(1), claim.clone()));
        assert_noop!(
            PoeModule::revoke_proof(Origin::signed(2), claim.clone()),
            Error::<Test>::NotProofOwner
        );
    })
}

#[test]
fn transfer_proof_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_proof(Origin::signed(1), claim.clone()));
        assert_eq!(Proofs::<Test>::get(&claim).unwrap(), (1, 0));
        assert_ok!(PoeModule::transfer_proof(Origin::signed(1), 2, claim.clone()));
        assert_eq!(Proofs::<Test>::get(&claim).unwrap(), (2, 0));
    })
}

#[test]
fn transfer_proof_failed_when_caller_is_not_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_proof(Origin::signed(1), claim.clone()));
        assert_noop!(
            PoeModule::transfer_proof(Origin::signed(2), 3, claim.clone()),
            Error::<Test>::NotProofOwner
        );
    })
}

#[test]
fn transfer_proof_failed_when_proof_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_noop!(
            PoeModule::transfer_proof(Origin::signed(1), 2, claim.clone()),
            Error::<Test>::ProofNotExist
        );
    })
}
