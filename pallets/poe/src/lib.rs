#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type ProofLimit: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub type Proofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>,
        (T::AccountId, T::BlockNumber)
    >;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProofCreated(T::AccountId, Vec<u8>),
        ProofRevoked(T::AccountId, Vec<u8>),
        ProofTransferred(T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExist,
        ProofNotExist,
        NotProofOwner,
        ProofExceedsLengthLimit,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_proof(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResultWithPostInfo {
            if proof.len() > T::ProofLimit::get() as usize {
                return Err(Error::<T>::ProofExceedsLengthLimit.into())
            }

            ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyExist);

            let sender = ensure_signed(origin)?;
            Proofs::<T>::insert(
                &proof,
                (sender.clone(), frame_system::Pallet::<T>::block_number()),
            );
            Self::deposit_event(Event::ProofCreated(sender, proof));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn revoke_proof(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let (owner, _) = Proofs::<T>::get(&proof)
                .ok_or(Error::<T>::ProofNotExist)?;

            ensure!(owner == sender, Error::<T>::NotProofOwner);

            Proofs::<T>::remove(&proof);
            Self::deposit_event(Event::ProofRevoked(sender, proof));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn transfer_proof(origin: OriginFor<T>, to: T::AccountId, proof: Vec<u8>)
                              -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let (owner, _) = Proofs::<T>::get(&proof)
                .ok_or(Error::<T>::ProofNotExist)?;

            ensure!(owner == sender, Error::<T>::NotProofOwner);

            Proofs::<T>::insert(
                &proof,
                (to.clone(), frame_system::Pallet::<T>::block_number()),
            );
            Self::deposit_event(Event::ProofTransferred(to, proof));

            Ok(().into())
        }
    }
}