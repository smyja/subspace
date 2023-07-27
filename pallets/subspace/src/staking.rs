use super::*;
use substrate_fixed::types::{I64F64, I32F32};
// import vec
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> { 


    //
	pub fn do_add_stake(
        origin: T::RuntimeOrigin, 
        netuid: u16,
        amount: u64
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId key information.
        let key = ensure_signed( origin )?;
        

		// --- 1. Ensure we don't exceed tx rate limit
		// ensure!( !Self::exceeds_tx_rate_limit(&key), Error::<T>::TxRateLimitExceeded);

        
        log::info!("do_add_stake( origin:{:?} stake_to_be_added:{:?} )", key, amount );
        
        ensure!( Self::can_remove_balance_from_account( &key, amount ), Error::<T>::NotEnoughBalanceToStake );

        Self::add_stake_on_account(netuid, &key, amount );
 
        // --- 5. Emit the staking event.
        log::info!("StakeAdded( key:{:?}, stake_to_be_added:{:?} )", key, amount );
        Self::deposit_event( Event::StakeAdded( key, amount ) );

        // --- 6. Ok and return.
        Ok(())
    }


    //
	pub fn do_add_delegate_stake(
        origin: T::RuntimeOrigin, 
        netuid: u16,
        to: T::AccountId,
        amount: u64
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId key information.
        let key = ensure_signed( origin )?;
        

        ensure!( Self::is_registered( netuid, &key.clone() ), Error::<T>::NotRegistered );  
        ensure!( Self::can_remove_balance_from_account( &key, amount ), Error::<T>::NotEnoughBalanceToStake );
        let uid: u16 = Self::get_uid_for_key( netuid, &key.clone() );
        Self::add_delegate_stake_on_account(netuid, &key, uid, amount );
        Ok(())
    }


        //
	pub fn do_remove_delegate_stake(
        origin: T::RuntimeOrigin, 
        netuid: u16,
        uid: u16,
        amount: u64
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId key information.
        let key = ensure_signed( origin )?;
        

		// --- 1. Ensure we don't exceed tx rate limit
		// ensure!( !Self::exceeds_tx_rate_limit(&key), Error::<T>::TxRateLimitExceeded);

        ensure!( Self::is_registered( netuid, &key.clone() ), Error::<T>::NotRegistered );  

        log::info!("do_add_stake( origin:{:?} stake_to_be_added:{:?} )", key, amount );
        
        ensure!( Self::can_remove_balance_from_account( &key, amount ), Error::<T>::NotEnoughBalanceToStake );

        Self::remove_delegate_stake_on_account(netuid, &key, uid, amount );
        // --- 6. Ok and return.amount
        Ok(())
    }





    pub fn do_remove_stake(
        origin: T::RuntimeOrigin, 
        netuid: u16,
        amount: u64
    ) -> dispatch::DispatchResult {

        // --- 1. We check the transaction is signed by the caller and retrieve the T::AccountId key information.
        let key = ensure_signed( origin )?;
        log::info!("do_remove_stake( origin:{:?} stake_to_be_removed:{:?} )", key, amount );


		// --- 6. Ensure we don't exceed tx rate limit
		// ensure!( !Self::exceeds_tx_rate_limit(&key), Error::<T>::TxRateLimitExceeded );

        // --- 5. Ensure that we can conver this u64 to a balance.
        ensure!( Self::has_enough_stake(netuid, &key, amount ), Error::<T>::NotEnoughStaketoWithdraw );
        let stake_to_be_added_as_currency = Self::u64_to_balance( amount );
        ensure!( stake_to_be_added_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance );

        // --- 7. We remove the balance from the key.
        Self::remove_stake_on_account(netuid,  &key, amount );

        // --- 9. Emit the unstaking event.
        log::info!("StakeRemoved( key:{:?}, stake_to_be_removed:{:?} )", key, amount );
        Self::deposit_event( Event::StakeRemoved( key, amount ) );

        // --- 10. Done and ok.
        Ok(())
    }


    // Returns the total amount of stake in the staking table.
    //
    pub fn get_total_subnet_stake(netuid:u16) -> u64 { 
        return SubnetTotalStake::<T>::get(netuid);
    }
    pub fn get_total_stake() -> u64 { 
        return TotalStake::<T>::get();
    }




    // Returns the stake under the cold - hot pairing in the staking table.
    //
    pub fn get_stake(netuid:u16, key: &T::AccountId ) -> u64 { 
        return Stake::<T>::get(netuid,  key );
    }
    

    pub fn key_account_exists(netuid:u16, key : &T::AccountId) -> bool {
        return Uids::<T>::contains_key(netuid, &key) ; 
    }

    // Returns true if the cold-hot staking account has enough balance to fufil the amount.
    //
    pub fn has_enough_stake(netuid: u16, key: &T::AccountId, amount: u64 ) -> bool {
        return Self::get_stake(netuid ,  key ) >= amount;
    }




    // Increases the stake on the cold - hot pairing by amount while also amounting other counters.
    // This function should be called rather than set_stake under account.
    // 

    pub fn add_stake_on_account(netuid: u16, key: &T::AccountId, amount: u64 ) -> bool{

        if !Stake::<T>::contains_key(netuid, key) {
            Stake::<T>::insert(netuid, key, 0);
        }


        Self::remove_balance_from_account( key, Self::u64_to_balance( amount ).unwrap() );
        Self::increase_stake_on_account(netuid, key, amount);
        
        return true;

    }


    pub fn get_delegate_to_stake_vector(netuid:u16, key:&T::AccountId, ) -> Vec<(u16, u64)> { 
        return DelegateToStake::<T>::get(netuid, key);
    }

    pub fn set_delegate_to_stake_vector(netuid:u16, key:&T::AccountId, delegate_to_stake_vector: Vec<(u16, u64)>) { 
        
        // we want to remove any keys that have a stake of 0, as these are from outside the subnet and can bloat the chain
        if delegate_to_stake_vector.len() == 0 {
            DelegateToStake::<T>::remove(netuid, key);
            return;
        }
        DelegateToStake::<T>::insert(netuid, key, delegate_to_stake_vector);
    }


    pub fn set_delegate_from_stake_vector(netuid:u16, uid: u16, delegate_from_stake_vector: Vec<(T::AccountId, u64)>) { 
        DelegateFromStake::<T>::insert(netuid, uid, delegate_from_stake_vector);
    }

    pub fn get_delegate_from_stake_vector(netuid:u16, uid: u16 ) -> Vec<(T::AccountId, u64)> { 
        
        return DelegateFromStake::<T>::get(netuid, uid).into_iter().collect::<Vec<(T::AccountId, u64)>>();
    }
    pub fn get_total_delegate_from_stake(netuid:u16, uid: u16 ) ->  u64 { 
        let delegate_from_stake_vector: Vec<(T::AccountId, u64)> = Self::get_delegate_from_stake_vector(netuid, uid);
        let mut total_delegate_from_stake: u64 = 0;
        for (k, v) in delegate_from_stake_vector {
            total_delegate_from_stake += v;
        }
        return total_delegate_from_stake;
    }
    pub fn get_total_delegate_to_stake(netuid:u16, key:&T::AccountId, ) -> u64 { 
        let mut delegate_to_stake_vector: Vec<(u16, u64)> = Self::get_delegate_to_stake_vector(netuid, key);
        let mut total_delegate_to_stake: u64 = 0;
        for (k, v) in delegate_to_stake_vector {
            total_delegate_to_stake += v;
        }
        let module_stake: u64 = Self::get_stake(netuid, key);
        return total_delegate_to_stake;
    }


    pub fn get_ownership_for_uid(netuid:u16, uid: u16 ) -> Vec<(T::AccountId, I64F64)> { 
        
        let delegate_from_stake_vector: Vec<(T::AccountId, u64)> = Self::get_delegate_from_stake_vector(netuid, uid);

        let mut total_delegate_from_stake: I64F64 = I64F64::from_num(0);
        let key_for_uid:T::AccountId  = Self::get_key_for_uid(netuid, uid);
        let module_stake: I64F64  = I64F64::from_num(Self::get_stake_for_uid(netuid, uid));

        let mut ownership_vector: Vec<(T::AccountId, I64F64)> = Vec::new();
        for (k, v) in delegate_from_stake_vector.clone().into_iter() {
            let ownership = I64F64::from_num(v) ;
            ownership_vector.push( (k.clone(), ownership) );
            total_delegate_from_stake += ownership;
        }
        ownership_vector.push( (key_for_uid, module_stake - total_delegate_from_stake) );

        if total_delegate_from_stake == I64F64::from_num(0) {
            return Vec::new();
        }

        for (k, v) in ownership_vector.clone() {
            let ownership = I64F64::from_num(v) / total_delegate_from_stake;
            ownership_vector.push( (k.clone(), ownership) );
        }

        return ownership_vector;
    }
    pub fn get_ownership_emission_for_key(netuid: u16, key: &T::AccountId, emission: u64) -> Vec<(T::AccountId, u64)> { 
        let uid : u16= Self::get_uid_for_key(netuid, key);
        return Self::get_ownership_emission_for_uid(netuid, uid, emission);
    }


    pub fn get_ownership_emission_for_uid(netuid:u16, uid:u16, emission:u64 ) -> Vec<(T::AccountId, u64)> { 
            
        let delegate_from_stake_vector: Vec<(T::AccountId, I64F64)> = Self::get_ownership_for_uid(netuid, uid);
        let mut emission_vector: Vec<(T::AccountId, u64)> = Vec::new();

        for (k, v) in delegate_from_stake_vector {
            let emission_for_delegate = (v * I64F64::from_num(emission)).floor().to_num::<u64>();
            emission_vector.push( (k, emission_for_delegate) );
        }

        return emission_vector;
    }

    pub fn add_delegate_stake_on_account(netuid: u16, key: &T::AccountId, uid: u16, amount: u64 ) -> bool{
        Self::increase_delegate_stake(netuid, key, uid, amount);
        Self::remove_balance_from_account( key, Self::u64_to_balance( amount ).unwrap() );
        
        return true;

    }
    pub fn remove_delegate_stake_on_account(netuid: u16, key: &T::AccountId, uid: u16, amount: u64 ) -> bool{
        Self::decrease_delegate_stake(netuid, key, uid, amount);
        Self::add_balance_to_account( key, Self::u64_to_balance( amount ).unwrap() );
        
        return true;

    }

    pub fn increase_delegate_stake(netuid: u16, key: &T::AccountId, uid: u16, amount: u64 ) -> bool{

        let mut delegete_to_stake_vector: Vec<(u16, u64)> = Self::get_delegate_to_stake_vector(netuid, key);
        let mut delegete_from_stake_vector: Vec<(T::AccountId, u64)> = Self::get_delegate_from_stake_vector(netuid, uid);
        let mut found_key_in_delegate_from_vector:bool= false;
        for (i, (k, v)) in delegete_from_stake_vector.clone().iter().enumerate() {
            let k_key : T::AccountId = k.clone();
            if *k == *key {
                delegete_from_stake_vector[i] = (k_key.clone(), *v + amount);
                found_key_in_delegate_from_vector = true;
            }
        }
        if !found_key_in_delegate_from_vector {
            delegete_from_stake_vector.push( (key.clone(), amount) );
        }

        let mut found_key_in_delegate_to_vector:bool= false;

        for (i, (k_uid, v)) in delegete_to_stake_vector.clone().iter().enumerate() {
            if *k_uid == uid {
                delegete_to_stake_vector[i] = (*k_uid, v + amount);
                found_key_in_delegate_to_vector = true;
            }
        }

        if !found_key_in_delegate_to_vector {
            delegete_to_stake_vector.push( (uid, amount) );
        }

        Self::set_delegate_to_stake_vector(netuid, key, delegete_to_stake_vector);
        Self::set_delegate_from_stake_vector(netuid, uid, delegete_from_stake_vector);
        Self::increase_stake_on_account(netuid, key, amount);
        
        return true;

    }



    pub fn decrease_delegate_stake(netuid: u16, key: &T::AccountId, uid: u16, amount: u64 ) -> bool{

        // FROM DELEGATE STAKE
        let mut delegete_to_stake_vector: Vec<(u16, u64)> = Self::get_delegate_to_stake_vector(netuid, key);
        let mut delegete_from_stake_vector: Vec<(T::AccountId, u64)> = Self::get_delegate_from_stake_vector(netuid, uid).clone();

        let mut idx_to_replace:usize = usize::MAX;
        let mut end_idx:usize = delegete_from_stake_vector.len() - 1;
        for (i, (k, v)) in delegete_from_stake_vector.clone().iter().enumerate() {
            let k_key : &T::AccountId = k;
            if *k == *key {
                let remaining_stake: u64 = *v - amount;
                delegete_from_stake_vector[i] = (k_key.clone(), remaining_stake);
                if remaining_stake == 0 {
                    idx_to_replace = i;
                }

            }
        }

        

        if idx_to_replace != usize::MAX {
            delegete_from_stake_vector[idx_to_replace] = delegete_from_stake_vector[end_idx].clone();
            delegete_from_stake_vector.remove(idx_to_replace);
        }



        // TO DELEGATE STAKE 
        idx_to_replace = usize::MAX;
        end_idx = delegete_to_stake_vector.len() - 1;

        for (i, (k_uid, v)) in delegete_to_stake_vector.clone().iter().enumerate() {
            if *k_uid == uid {
                let remaining_stake: u64 = *v - amount;
                delegete_to_stake_vector[i] = (*k_uid, remaining_stake);
                if remaining_stake == 0 {
                    idx_to_replace = i;
                }
            }
        }

        if idx_to_replace != usize::MAX {
            delegete_to_stake_vector[idx_to_replace] = delegete_to_stake_vector[end_idx].clone();
            delegete_to_stake_vector.remove(idx_to_replace);
        }

        // if !found_key_in_delegate_to_vector {
        Self::set_delegate_to_stake_vector(netuid, key, delegete_to_stake_vector);
        Self::set_delegate_from_stake_vector(netuid, uid, delegete_from_stake_vector);
        Self::add_balance_to_account( key, Self::u64_to_balance( amount ).unwrap() );
        
        return true;

    }
    pub fn remove_delegate_stake_from_storage(netuid: u16, key: &T::AccountId) {
        let uid = Self::get_uid_for_key(netuid, key);
        Self::remove_delegate_stake_from_storage_for_uid(netuid, uid);

    }
    

    pub fn remove_delegate_stake_from_storage_for_uid(netuid: u16, uid: u16 ) -> bool{

        let mut delegete_from_stake_vector: Vec<(T::AccountId, u64)> = Self::get_delegate_from_stake_vector(netuid, uid);

        for (i, (key, amount)) in delegete_from_stake_vector.iter().enumerate() {
            Self::remove_delegate_stake_on_account(netuid, key, uid, *amount);
            
        }
        DelegateFromStake::<T>::remove(netuid, uid);

        
        return true;
    }





    pub fn increase_stake_on_account(netuid:u16, key: &T::AccountId, amount: u64 ){
        Stake::<T>::insert(netuid, key, Stake::<T>::get(netuid, key).saturating_add( amount ) );
        SubnetTotalStake::<T>::insert(netuid , SubnetTotalStake::<T>::get(netuid).saturating_add( amount ) );
        TotalStake::<T>::put(TotalStake::<T>::get().saturating_add( amount ) );

    }

    // Decreases the stake on the cold - hot pairing by the amount while decreasing other counters.
    //
    pub fn decrease_stake_on_account(netuid:u16, key: &T::AccountId, amount: u64 ) {
        // --- 8. We add the balancer to the key.  If the above fails we will not credit this key.
        Stake::<T>::insert( netuid, key, Stake::<T>::get(netuid,  key).saturating_sub( amount ) );
        TotalStake::<T>::put(TotalStake::<T>::get().saturating_sub( amount ) );
        SubnetTotalStake::<T>::insert(netuid, SubnetTotalStake::<T>::get(netuid).saturating_sub( amount ) );
    }
    // Decreases the stake on the cold - hot pairing by the amount while decreasing other counters.
    //
    pub fn remove_stake_on_account(netuid:u16, key: &T::AccountId, amount: u64 ) {

        let stake_to_be_added_as_currency = Self::u64_to_balance( amount );

        // --- 8. We add the balancer to the key.  If the above fails we will not credit this key.
        Self::decrease_stake_on_account(netuid, &key, amount );
        Self::add_balance_to_account( &key, stake_to_be_added_as_currency.unwrap() );
    }

    // Decreases the stake on the cold - hot pairing by the amount while decreasing other counters.
    //
    pub fn remove_all_stake_on_account(netuid:u16, key: &T::AccountId ) {

        let amount = Stake::<T>::get(netuid,  &key);
        Self::remove_stake_on_account(netuid, &key, amount );
    }

    // Decreases the stake on the cold - hot pairing by the amount while decreasing other counters.
    //
    pub fn remove_stake_from_storage(netuid:u16, key: &T::AccountId ) {

        Self::remove_all_stake_on_account(netuid, &key );
        Stake::<T>::remove(netuid, &key);
    }

	pub fn u64_to_balance( input: u64 ) -> Option<<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance> { input.try_into().ok() }

    pub fn add_balance_to_account(key: &T::AccountId, amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance) {
        T::Currency::deposit_creating(&key, amount); // Infallibe
    }

    pub fn set_balance_on_account(key: &T::AccountId, amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance) {
        T::Currency::make_free_balance_be(&key, amount); 
    }

    pub fn can_remove_balance_from_account(key: &T::AccountId, amount_64: u64) -> bool {
        let amount_as_balance = Self::u64_to_balance( amount_64 );
        if amount_as_balance.is_none() {
            return false;
        }
        let amount = amount_as_balance.unwrap();
        let current_balance = Self::get_balance(key);
        if amount > current_balance {
            return false;
        }
        // This bit is currently untested. @todo
        let new_potential_balance = current_balance - amount;
        let can_withdraw = T::Currency::ensure_can_withdraw(&key, amount, WithdrawReasons::except(WithdrawReasons::TIP), new_potential_balance).is_ok();
        can_withdraw
    }

    pub fn get_balance(key: &T::AccountId) -> <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance {
        return T::Currency::free_balance(&key);
    }

    pub fn balance_to_u64( input: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance) -> u64 { input.try_into().ok().unwrap() }

    pub fn get_balance_as_u64(key: &T::AccountId) -> u64 {
        return Self::balance_to_u64( Self::get_balance(key) );
    }

    pub fn has_enough_balance(key: &T::AccountId, amount: u64 ) -> bool {
        return Self::get_balance_as_u64(key) >= amount;
    }

    pub fn resolve_stake_amount(key: &T::AccountId, stake: u64 ) -> u64 {
        let balance = Self::get_balance_as_u64(key);
        if balance < stake {
            return balance;
        } else {
            return stake;
        }
    }


    pub fn remove_balance_from_account(key: &T::AccountId, amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance) -> bool {
        return match T::Currency::withdraw(&key, amount, WithdrawReasons::except(WithdrawReasons::TIP), ExistenceRequirement::KeepAlive) {
            Ok(_result) => {
                true
            }
            Err(_error) => {
                false
            }
        };
    }

}