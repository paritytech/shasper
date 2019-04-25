use bm::{MerkleDB, MerkleVec, MerkleTuple, EndOf, IntermediateOf, IntermediateSizeOf, ValueOf};
use ssz::Digestible;
use generic_array::{GenericArray, ArrayLength};

use core::marker::PhantomData;

pub trait ShadowDB {
	type KeySize: ArrayLength<u8>;
	type Value;

	fn get(&self, key: &GenericArray<u8, Self::KeySize>) -> Option<Self::Value>;
	fn insert(&mut self, key: GenericArray<u8, Self::KeySize>, value: Self::Value);
	fn remove(&mut self, key: &GenericArray<u8, Self::KeySize>) -> Option<Self::Value>;
}

pub struct ShadowVec<DB: MerkleDB, SDB: ShadowDB, T> {
	vec: MerkleVec<DB>,
	_marker: PhantomData<(SDB, T)>,
}

impl<DB: MerkleDB, SDB, T> ShadowVec<DB, SDB, T> where
	EndOf<DB>: From<IntermediateOf<DB>> + Into<IntermediateOf<DB>> + From<usize> + Into<usize>,
	SDB: ShadowDB<KeySize=IntermediateSizeOf<DB>, Value=T>,
	T: Digestible<DB::Digest>,
{
    /// Push a new value to the vector.
    pub fn push(&mut self, db: &mut DB, sdb: &mut SDB, value: T) {
		let hashed = Digestible::<DB::Digest>::hash(&value);
		sdb.insert(hashed.clone(), value);
		self.vec.push(db, hashed.into());
	}

	/// Pop a value from the vector.
    pub fn pop(&mut self, db: &mut DB, sdb: &mut SDB) -> Option<T> {
		self.vec.pop(db).and_then(|hashed| {
			sdb.remove(&hashed.into())
		})
    }

	/// Set value at index.
    pub fn set(&mut self, db: &mut DB, sdb: &mut SDB, index: usize, value: T) {
		let hashed = Digestible::<DB::Digest>::hash(&value);
		sdb.insert(hashed.clone(), value);
		self.vec.set(db, index, hashed.into());
    }

	/// Get value at index.
    pub fn get(&self, db: &DB, sdb: &SDB, index: usize) -> T {
		let hashed = self.vec.get(db, index);
		sdb.get(&hashed.into()).expect("Hash must exist")
	}

	/// Root of the current merkle vector.
    pub fn root(&self) -> ValueOf<DB> {
        self.vec.root()
    }

	/// Length of the vector.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Create a new vector.
    pub fn create(db: &mut DB) -> Self {
		Self {
			vec: MerkleVec::create(db),
			_marker: PhantomData,
		}
    }

    /// Drop the current vector.
    pub fn drop(mut self, db: &mut DB, sdb: &mut SDB) {
		while let Some(hashed) = self.vec.pop(db) {
			sdb.remove(&hashed.into());
		}
		self.vec.drop(db)
    }

    /// Leak the current vector.
    pub fn leak(self) -> (ValueOf<DB>, ValueOf<DB>, ValueOf<DB>, usize) {
		self.vec.leak()
    }

    /// Initialize from a previously leaked one.
    pub fn from_leaked(raw_root: ValueOf<DB>, tuple_root: ValueOf<DB>, empty_root: ValueOf<DB>, len: usize) -> Self {
		Self {
			vec: MerkleVec::from_leaked(raw_root, tuple_root, empty_root, len),
			_marker: PhantomData,
		}
    }
}

pub struct ShadowTuple<DB: MerkleDB, SDB: ShadowDB, T> {
	tuple: MerkleTuple<DB>,
	_marker: PhantomData<(SDB, T)>,
}

impl<DB: MerkleDB, SDB, T> ShadowTuple<DB, SDB, T> where
	EndOf<DB>: From<IntermediateOf<DB>> + Into<IntermediateOf<DB>>,
	SDB: ShadowDB<KeySize=IntermediateSizeOf<DB>, Value=T>,
	T: Digestible<DB::Digest>,
{
    /// Push a new value to the tuple.
    pub fn push(&mut self, db: &mut DB, sdb: &mut SDB, value: T) {
		let hashed = Digestible::<DB::Digest>::hash(&value);
		sdb.insert(hashed.clone(), value);
		self.tuple.push(db, hashed.into());
	}

	/// Pop a value from the tuple.
    pub fn pop(&mut self, db: &mut DB, sdb: &mut SDB) -> Option<T> {
		self.tuple.pop(db).and_then(|hashed| {
			sdb.remove(&hashed.into())
		})
    }

	/// Set value at index.
    pub fn set(&mut self, db: &mut DB, sdb: &mut SDB, index: usize, value: T) {
		let hashed = Digestible::<DB::Digest>::hash(&value);
		sdb.insert(hashed.clone(), value);
		self.tuple.set(db, index, hashed.into());
    }

	/// Get value at index.
    pub fn get(&self, db: &DB, sdb: &SDB, index: usize) -> T {
		let hashed = self.tuple.get(db, index);
		sdb.get(&hashed.into()).expect("Hash must exist")
	}

	/// Root of the current merkle tuple.
    pub fn root(&self) -> ValueOf<DB> {
        self.tuple.root()
    }

	/// Length of the tuple.
    pub fn len(&self) -> usize {
        self.tuple.len()
    }

    /// Create a new tuple.
    pub fn create(db: &mut DB) -> Self {
		Self {
			tuple: MerkleTuple::create(db, 0),
			_marker: PhantomData,
		}
    }

    /// Drop the current tuple.
    pub fn drop(mut self, db: &mut DB, sdb: &mut SDB) {
		while let Some(hashed) = self.tuple.pop(db) {
			sdb.remove(&hashed.into());
		}
		self.tuple.drop(db)
    }

    /// Leak the current tuple.
    pub fn leak(self) -> (ValueOf<DB>, ValueOf<DB>, usize) {
		self.tuple.leak()
    }

    /// Initialize from a previously leaked one.
    pub fn from_leaked(tuple_root: ValueOf<DB>, empty_root: ValueOf<DB>, len: usize) -> Self {
		Self {
			tuple: MerkleTuple::from_leaked(tuple_root, empty_root, len),
			_marker: PhantomData,
		}
    }
}
