#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod mock;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Randomness, ReservableCurrency, Currency, ExistenceRequirement},
        sp_runtime::traits::AtLeast32Bit,
    };
    use frame_system::pallet_prelude::*;
    use codec::{Encode, Decode};
    use sp_io::hashing::blake2_128;
    use sp_runtime::traits::Bounded;

    #[derive(Encode, Decode)]
    pub struct Kitty(pub [u8; 16]);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type KittyIndex: Parameter + Member + AtLeast32Bit + Default + Copy;
        type Currency: ReservableCurrency<Self::AccountId>;
        type PledgeQuantity: Get<BalanceOf<Self>>;
    }

    type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::KittyIndex,
        Option<Kitty>,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittyOwners<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::KittyIndex,
        Option<T::AccountId>,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn kitty_price)]
    pub type KittyPrice<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::KittyIndex,
        Option<BalanceOf<T>>,
        ValueQuery
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreated(T::AccountId, T::KittyIndex),
        KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
        KittyAsk(T::KittyIndex, Option<BalanceOf<T>>),
        KittySold(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        KittiesCountOverflow,
        NotKittyOwner,
        SameKitties,
        InvalidKittyId,
        KittyNotForSale,
        PriceTooLow,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;
            let kitty_id = Self::create_kitty(
                owner.clone(),
                Self::random_value(&owner)
            )?;
            Self::deposit_event(Event::KittyCreated(owner, kitty_id));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, kitty_id: T::KittyIndex)
            -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
            ensure!(Some(sender.clone()) == KittyOwners::<T>::get(kitty_id), Error::<T>::NotKittyOwner);

            Self::do_transfer(sender.clone(), to.clone(), kitty_id)?;

            Self::deposit_event(
                Event::KittyTransferred(sender, to, kitty_id)
            );

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn breed(origin: OriginFor<T>, parent_id_m: T::KittyIndex, parent_id_f: T::KittyIndex)
            -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;
            ensure!(parent_id_m != parent_id_f, Error::<T>::SameKitties);

            let kitty_m = Self::kitties(parent_id_m).ok_or(Error::<T>::InvalidKittyId)?;
            let kitty_f = Self::kitties(parent_id_f).ok_or(Error::<T>::InvalidKittyId)?;

            let dna_m = kitty_m.0;
            let dna_f = kitty_f.0;

            let selector = Self::random_value(&owner);
            let mut dna = [0u8; 16];

            for i in 0..dna_m.len() {
                dna[i] = (selector[i] & dna_m[i]) | (!selector[i] & dna_f[i])
            }

            let kitty_id = Self::create_kitty(owner.clone(), dna)?;
            Self::deposit_event(
                Event::KittyCreated(owner, kitty_id)
            );

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn ask(origin: OriginFor<T>, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>)
            -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
            ensure!(KittyOwners::<T>::get(kitty_id) == Some(sender), Error::<T>::NotKittyOwner);

            KittyPrice::<T>::mutate_exists(
                kitty_id,
                |old_price| *old_price = Some(price.clone())
            );

            Self::deposit_event(
              Event::KittyAsk(kitty_id, price)
            );

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex, price: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            let kitty_price = Self::kitty_price(kitty_id).ok_or(Error::<T>::KittyNotForSale)?;

            ensure!(price >= kitty_price, Error::<T>::PriceTooLow);

            T::Currency::transfer(
                &sender,
                &owner,
                kitty_price,
                #[cfg(test)]
                ExistenceRequirement::AllowDeath,
                #[cfg(not(test))]
                ExistenceRequirement::KeepAlive,
            )?;
            KittyPrice::<T>::remove(kitty_id);
            KittyOwners::<T>::insert(kitty_id, Some(sender.clone()));

            Self::do_transfer(owner.clone(), sender.clone(), kitty_id)?;

            Self::deposit_event(
                Event::<T>::KittySold(
                    owner,
                    sender,
                    kitty_id,
                    kitty_price
                )
            );

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn next_kitty_id() -> Result<T::KittyIndex, Error<T>> {
            let kitty_id = Self::kitties_count().unwrap_or(0_u32.into());
            if kitty_id == T::KittyIndex::max_value() {
                return Err(Error::<T>::KittiesCountOverflow)
            }
            return Ok(kitty_id)
        }

        fn do_transfer(from: T::AccountId, to: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
            let pledge = T::PledgeQuantity::get();
            T::Currency::reserve(&to, pledge)?;
            T::Currency::unreserve(&from, pledge);

            KittyOwners::<T>::insert(kitty_id, Some(to.clone()));

            Ok(())
        }

        fn create_kitty(owner: T::AccountId, dna: [u8; 16]) -> Result<T::KittyIndex, DispatchError> {
            let kitty_id = Self::next_kitty_id()?;

            let pleage = T::PledgeQuantity::get();
            T::Currency::reserve(&owner, pleage)?;

            Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
            KittyOwners::<T>::insert(kitty_id, Some(owner.clone()));
            KittiesCount::<T>::put(kitty_id + 1_u32.into());

            Ok(kitty_id)
        }

        fn random_value(sender: &T::AccountId) -> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );
            payload.using_encoded(blake2_128)
        }
    }
}
