use support::{decl_module, decl_storage, ensure, StorageValue, StorageMap, dispatch::Result, Parameter};
use sr_primitives::traits::{SimpleArithmetic, Bounded, CheckedAdd, CheckedSub};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;
use rstd::result;

pub trait Trait: system::Trait {
	type KittyIndex: Parameter + SimpleArithmetic + Bounded + Default + Copy;
}

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		pub Kitties get(kitty): map T::KittyIndex => Option<Kitty>;
		/// Stores the total number of kitties. i.e. the next kitty index
		pub KittiesCount get(kitties_count): T::KittyIndex;

		/// Get Kitty Owner Account by Kitty ID 
		pub KittyOwner get(owner_of): map T::KittyIndex => Option<T::AccountId>;
		/// Get kitty ID by account ID and user kitty index
		pub OwnedKitties get(owned_kitties): map (T::AccountId, T::KittyIndex) => T::KittyIndex;
		/// Get number of kitties by account ID
		pub OwnedKittiesCount get(owned_kitties_count): map T::AccountId => T::KittyIndex;
		/// Get user's kitty ID from kitty ID
		pub OwnedKittiesIndex: map T::KittyIndex => T::KittyIndex;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Create a new kitty
		/// 作业：重构create方法，避免重复代码
		pub fn create(origin) {

			// check msg sender
			let sender = ensure_signed(origin)?;

			// using internal method to create a new kitty
			Self::create_kitty(sender)
		}

		/// Breed kitties
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_breed(sender, kitty_id_1, kitty_id_2)?;
		}

		pub fn transfer_kitty(origin, T::Account, kitty_id: T::KittyIndex) -> Result {
			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(kitty_id).ok_or("Owner of this kitty not found.")?;

			//check msg sender is the kitty owner
			ensure!(owner == sender,"Kitty owner invalid.");

			Self::transfer_kitty(sender,to,kitty_id)
		}
	}
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	// 作业：实现combine_dna
	// 伪代码：
	// selector.map_bits(|bit, index| if (bit == 1) { dna1 & (1 << index) } else { dna2 & (1 << index) })
	// 注意 map_bits这个方法不存在。只要能达到同样效果，不局限算法
	// 测试数据：dna1 = 0b11110000, dna2 = 0b11001100, selector = 0b10101010, 返回值 0b11100100
	return dna1;
}

impl<T: Trait> Module<T> {
	
	//using internal method to create kitties
	fn create_kitty(owner: T::AccountId) -> Result {

		// get new kitty id
		let new_kitty_id = Self::next_kitty_id()?;

		// using internal method to generate random DNA
		let dna = Self::random_value(&owner);

		// construct the new kitty
		let new_kitty = Kitty(dna);

		// using internal method to add the new kitty
		Self::insert_kitty(owner.clone(),new_kitty_id,new_kitty);

		Ok(())
	}

	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
		payload.using_encoded(blake2_128)
	}

	fn next_kitty_id() -> result::Result<T::KittyIndex, &'static str> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err("Kitties count overflow");
		}
		Ok(kitty_id)
	}

	fn insert_kitty(owner: T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
		// Create and store kitty
		<Kitties<T>>::insert(kitty_id, kitty);
		<KittiesCount<T>>::put(kitty_id + 1.into());

		// Store the ownership information
		let user_kitties_id = Self::owned_kitties_count(owner.clone());
		<OwnedKitties<T>>::insert((owner.clone(), user_kitties_id), kitty_id);
		<OwnedKittiesCount<T>>::insert(owner.clone(), user_kitties_id + 1.into());
		
		<OwnedKittiesIndex<T>>::insert(kitty_id, user_kitties_id);
		<KittyOwner<T>>::insert(kitty_id, owner.clone());
	}

	fn do_breed(sender: T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> Result {
		let kitty1 = Self::kitty(kitty_id_1);
		let kitty2 = Self::kitty(kitty_id_2);

		ensure!(kitty1.is_some(), "Invalid kitty_id_1");
		ensure!(kitty2.is_some(), "Invalid kitty_id_2");
		ensure!(kitty_id_1 != kitty_id_2, "Needs different parent");

		let kitty_id = Self::next_kitty_id()?;

		let kitty1_dna = kitty1.unwrap().0;
		let kitty2_dna = kitty2.unwrap().0;

		// Generate a random 128bit value
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		// Combine parents and selector to create new kitty
		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

		Ok(())
	}

	fn transfer_kitty(from: T::AccountId, to: T::AccountId, kitty_id: T::KittyIndex) -> Result {
		
		// check kitty owner 
		let owner = Self::owner_of(kitty_id).ok_or("Kitty Owner Invalid.")?;
		
		// check from account
		ensure!(owner == from,"from account is not the owner.");

		// get count from 'from' and 'to' account for calc
		let from_account_owned_kitties = Self::owned_kitties_count(&from);
		let to_account_owned_kitties = Self::owned_kitties_count(&to);

		// safe minus one for 'from' account owned index
		let new_from_account_owned_kitties = from_account_owned_kitties.checked_sub(1)
			.ok_or("transfer error of 'from' account.");

		// safe add one for 'to' account owned index
		let new_to_account_owned_kitties = from_account_owned_kitties.checked_add(1)
			.ok_or("transfer error of 'to' account.");

		// get kitty index 
		let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);

		if kitty_index != new_from_account_owned_kitties {
			let last_kitty_id = <OwnedKitties<T>>::get((from.clone(),new_from_account_owned_kitties));
			<OwnedKitties<T>>::insert((from.clone(),kitty_index),last_kitty_id);
			<OwnedKittiesIndex<T>>::insert(last_kitty_id,kitty_index);
		}
		
		// change kitty ownership
		<KittyOwner<T>>::insert(&kitty_id,&to);
		<OwnedKittiesIndex<T>>::insert(kitty_id,to_account_owned_kitties);
		
		// change kitty ownership
		<OwnedKitties<T>>::remove((from.clone(),new_from_account_owned_kitties));
		<OwnedKitties<T>>::insert((to.clone(),to_account_owned_kitties),kitty_id);

		// change owner kitties counter
		<OwnedKittiesCount<T>>::insert(&from,new_from_account_owned_kitties);
		<OwnedKittiesCount<T>>::insert(&to,new_to_account_owned_kitties);

		// done
		Ok(())
	}
}