use crate::*;
use frame_support::{traits::Get, traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use codec::Decode;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct FixFarmingPolicy<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for FixFarmingPolicy<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V10Struct);

        let farms_count: u64 = Farms::<T>::iter().count() as u64;
        log::info!(
            "🔎 FixFarmingPolicy pre migration: Number of existing farms {:?}",
            farms_count
        );

        info!("👥  TFGrid pallet to V11 passes PRE migrate checks ✅",);
        Ok(farms_count.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V10Struct {
            fix_farming_policy_migration_::<T>()
        } else {
            info!(" >>> Unused TFGrid pallet V11 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_farms_count: Vec<u8>) -> Result<(), &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V11Struct);

        // Check number of farms against pre-check result
        let pre_farms_count: u64 = Decode::decode(&mut pre_farms_count.as_slice())
            .expect("the state parameter should be something that was generated by pre_upgrade");
        assert_eq!(
            Farms::<T>::iter().count() as u64,
            pre_farms_count,
            "Number of farms migrated does not match"
        );

        info!(
            "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
            Pallet::<T>::pallet_version()
        );

        Ok(())
    }
}

pub fn fix_farming_policy_migration_<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating farm storage...");

    let mut read_writes = 0;

    Farms::<T>::translate::<FarmInfoOf<T>, _>(|k, f| {
        let mut new_farm = f;

        new_farm.pricing_policy_id = 1;
        debug!("migrated farm: {:?}", k);

        read_writes += 1;
        Some(new_farm)
    });

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V11Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes, read_writes)
}