use crate::{Error, mock::*};
use super::*;
use frame_support::{assert_ok, assert_noop};

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_eq!(KittiesModule::kitties_count().unwrap(), 1);
        assert_eq!(KittyOwners::<Test>::get(0).unwrap(), 1);
        assert_eq!(Balances::reserved_balance(1), 1);
    });
}

#[test]
fn create_failed_when_kitty_index_overflow() {
    new_test_ext().execute_with(|| {
        KittiesCount::<Test>::put(u32::MAX);

        assert_noop!(
            KittiesModule::create(Origin::signed(1)),
            Error::<Test>::KittiesCountOverflow
        );
    })
}

#[test]
fn create_kitty_failed_when_not_enough_funds_pledged() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            KittiesModule::create(Origin::signed(6)),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    })
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::reserved_balance(2), 0);
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_eq!(Balances::reserved_balance(1), 1);
        assert_eq!(KittyOwners::<Test>::get(0).unwrap(), 1);
        assert_ok!(KittiesModule::transfer(Origin::signed(1), 2, 0));
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::reserved_balance(2), 1);
        assert_eq!(KittyOwners::<Test>::get(0).unwrap(), 2)
    })
}

#[test]
fn transfer_failed_when_caller_is_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_noop!(
            KittiesModule::transfer(Origin::signed(2), 3, 0),
            Error::<Test>::NotKittyOwner
        );
    })
}

#[test]
fn transfer_failed_when_recipient_dont_have_enougth_funds() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_eq!(Balances::reserved_balance(1), 1);
        assert_noop!(
            KittiesModule::transfer(Origin::signed(1), 6, 0),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    })
}

#[test]
fn breed_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 1
        assert_eq!(Balances::reserved_balance(1), 2);
        assert_ok!(KittiesModule::breed(Origin::signed(1), 0, 1)); // kitty_index: 2
        assert_eq!(Balances::reserved_balance(1), 3);
        assert_eq!(KittiesModule::kitties_count().unwrap(), 3)
    })
}

#[test]
fn breed_failed_when_kitty_not_exist() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_noop!(
            KittiesModule::breed(Origin::signed(1), 1, 2),
            Error::<Test>::InvalidKittyId
        );
    })
}

#[test]
fn breed_failed_when_parents_are_same() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_noop!(
            KittiesModule::breed(Origin::signed(1), 1, 1),
            Error::<Test>::SameKitties
        );
    })
}

#[test]
fn ask_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_eq!(Balances::reserved_balance(1), 1);
        assert_ok!(KittiesModule::ask(Origin::signed(1), 0, Some(10)));
        assert_eq!(KittyPrice::<Test>::get(0), Some(10));
    })
}

#[test]
fn ask_failed_when_kitty_not_exist() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            KittiesModule::ask(Origin::signed(1), 0, Some(10)),
            Error::<Test>::InvalidKittyId
        );
    })
}

#[test]
fn ask_failed_when_caller_is_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_noop!(
            KittiesModule::ask(Origin::signed(2), 0, Some(10)),
            Error::<Test>::NotKittyOwner
        );
    })
}

#[test]
fn buy_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::reserved_balance(2), 0);
        assert_eq!(Balances::free_balance(1), 20);
        assert_eq!(Balances::free_balance(2), 20);

        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_eq!(Balances::reserved_balance(1), 1);
        assert_ok!(KittiesModule::ask(Origin::signed(1), 0, Some(1)));
        assert_eq!(KittyOwners::<Test>::get(0), Some(1));
        assert_ok!(KittiesModule::buy(Origin::signed(2), 0, 1));

        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::reserved_balance(2), 1);
        assert_eq!(Balances::free_balance(1), 21);
        assert_eq!(Balances::free_balance(2), 18); // 20 - 1 - 1(reserved)

        assert_eq!(KittyOwners::<Test>::get(0), Some(2));
    })
}

#[test]
fn buy_failed_when_kitty_not_exist() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            KittiesModule::buy(Origin::signed(1), 0, 1),
            Error::<Test>::InvalidKittyId
        );
    })
}

#[test]
fn buy_failed_when_kitty_not_for_sale() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_noop!(
            KittiesModule::buy(Origin::signed(2), 0, 1),
            Error::<Test>::KittyNotForSale
        );
    })
}

#[test]
fn buy_failed_when_not_enough_funds () {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1))); // kitty_index: 0
        assert_ok!(KittiesModule::ask(Origin::signed(1), 0, Some(1)));
        assert_noop!(
            KittiesModule::buy(Origin::signed(6), 0, 1),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    })
}
