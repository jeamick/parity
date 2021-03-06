// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! State snapshotting tests.

use basic_account::BasicAccount;
use snapshot::account;
use snapshot::{chunk_state, Error as SnapshotError, Progress, StateRebuilder};
use snapshot::io::{PackedReader, PackedWriter, SnapshotReader, SnapshotWriter};
use super::helpers::{compare_dbs, StateProducer};

use error::Error;

use rand::{XorShiftRng, SeedableRng};
use util::hash::H256;
use util::journaldb::{self, Algorithm};
use util::kvdb::{Database, DatabaseConfig};
use util::memorydb::MemoryDB;
use util::Mutex;
use devtools::RandomTempPath;

use util::sha3::SHA3_NULL_RLP;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[test]
fn snap_and_restore() {
	let mut producer = StateProducer::new();
	let mut rng = XorShiftRng::from_seed([1, 2, 3, 4]);
	let mut old_db = MemoryDB::new();
	let db_cfg = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	for _ in 0..150 {
		producer.tick(&mut rng, &mut old_db);
	}

	let snap_dir = RandomTempPath::create_dir();
	let mut snap_file = snap_dir.as_path().to_owned();
	snap_file.push("SNAP");

	let state_root = producer.state_root();
	let writer = Mutex::new(PackedWriter::new(&snap_file).unwrap());

	let state_hashes = chunk_state(&old_db, &state_root, &writer, &Progress::default()).unwrap();

	writer.into_inner().finish(::snapshot::ManifestData {
		state_hashes: state_hashes,
		block_hashes: Vec::new(),
		state_root: state_root,
		block_number: 0,
		block_hash: H256::default(),
	}).unwrap();

	let mut db_path = snap_dir.as_path().to_owned();
	db_path.push("db");
	let db = {
		let new_db = Arc::new(Database::open(&db_cfg, &db_path.to_string_lossy()).unwrap());
		let mut rebuilder = StateRebuilder::new(new_db.clone(), Algorithm::Archive);
		let reader = PackedReader::new(&snap_file).unwrap().unwrap();

		let flag = AtomicBool::new(true);

		for chunk_hash in &reader.manifest().state_hashes {
			let raw = reader.chunk(*chunk_hash).unwrap();
			let chunk = ::util::snappy::decompress(&raw).unwrap();

			rebuilder.feed(&chunk, &flag).unwrap();
		}

		assert_eq!(rebuilder.state_root(), state_root);
		rebuilder.check_missing().unwrap();

		new_db
	};

	let new_db = journaldb::new(db, Algorithm::Archive, ::db::COL_STATE);

	compare_dbs(&old_db, new_db.as_hashdb());
}

#[test]
fn get_code_from_prev_chunk() {
	use std::collections::HashSet;
	use rlp::{RlpStream, Stream};
	use util::{HashDB, H256, FixedHash, U256, Hashable};

	use account_db::{AccountDBMut, AccountDB};

	let code = b"this is definitely code";
	let mut used_code = HashSet::new();
	let mut acc_stream = RlpStream::new_list(4);
	acc_stream.append(&U256::default())
		.append(&U256::default())
		.append(&SHA3_NULL_RLP)
		.append(&code.sha3());

	let (h1, h2) = (H256::random(), H256::random());

	// two accounts with the same code, one per chunk.
	// first one will have code inlined,
	// second will just have its hash.
	let thin_rlp = acc_stream.out();
	let acc: BasicAccount = ::rlp::decode(&thin_rlp);

	let mut make_chunk = |acc, hash| {
		let mut db = MemoryDB::new();
		AccountDBMut::from_hash(&mut db, hash).insert(&code[..]);

		let fat_rlp = account::to_fat_rlp(&acc, &AccountDB::from_hash(&db, hash), &mut used_code).unwrap();

		let mut stream = RlpStream::new_list(1);
		stream.begin_list(2).append(&hash).append_raw(&fat_rlp, 1);
		stream.out()
	};

	let chunk1 = make_chunk(acc.clone(), h1);
	let chunk2 = make_chunk(acc, h2);

	let db_path = RandomTempPath::create_dir();
	let db_cfg = DatabaseConfig::with_columns(::db::NUM_COLUMNS);
	let new_db = Arc::new(Database::open(&db_cfg, &db_path.to_string_lossy()).unwrap());

	let mut rebuilder = StateRebuilder::new(new_db, Algorithm::Archive);
	let flag = AtomicBool::new(true);

	rebuilder.feed(&chunk1, &flag).unwrap();
	rebuilder.feed(&chunk2, &flag).unwrap();

	rebuilder.check_missing().unwrap();
}

#[test]
fn checks_flag() {
	let mut producer = StateProducer::new();
	let mut rng = XorShiftRng::from_seed([5, 6, 7, 8]);
	let mut old_db = MemoryDB::new();
	let db_cfg = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	for _ in 0..10 {
		producer.tick(&mut rng, &mut old_db);
	}

	let snap_dir = RandomTempPath::create_dir();
	let mut snap_file = snap_dir.as_path().to_owned();
	snap_file.push("SNAP");

	let state_root = producer.state_root();
	let writer = Mutex::new(PackedWriter::new(&snap_file).unwrap());

	let state_hashes = chunk_state(&old_db, &state_root, &writer, &Progress::default()).unwrap();

	writer.into_inner().finish(::snapshot::ManifestData {
		state_hashes: state_hashes,
		block_hashes: Vec::new(),
		state_root: state_root,
		block_number: 0,
		block_hash: H256::default(),
	}).unwrap();

	let mut db_path = snap_dir.as_path().to_owned();
	db_path.push("db");
	{
		let new_db = Arc::new(Database::open(&db_cfg, &db_path.to_string_lossy()).unwrap());
		let mut rebuilder = StateRebuilder::new(new_db.clone(), Algorithm::Archive);
		let reader = PackedReader::new(&snap_file).unwrap().unwrap();

		let flag = AtomicBool::new(false);

		for chunk_hash in &reader.manifest().state_hashes {
			let raw = reader.chunk(*chunk_hash).unwrap();
			let chunk = ::util::snappy::decompress(&raw).unwrap();

			match rebuilder.feed(&chunk, &flag) {
				Err(Error::Snapshot(SnapshotError::RestorationAborted)) => {},
				_ => panic!("unexpected result when feeding with flag off"),
			}
		}
	}
}
