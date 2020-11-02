use crate::*;
use ethereum_types;
use rlp::{
	Decodable as RlpDecodable, DecoderError as RlpDecoderError, Encodable as RlpEncodable, Rlp,
	RlpStream,
};
use rlp_derive::{RlpDecodable as RlpDecodableDerive, RlpEncodable as RlpEncodableDerive};
use ethereum_types::{Bloom, H160, H256, H64, U256};

#[derive(Debug, Clone, Encode, Decode)]
pub struct BlockHeader {
	pub parent_hash: H256,
	pub uncles_hash: H256,
	pub author: H160,
	pub state_root: H256,
	pub transactions_root: H256,
	pub receipts_root: H256,
	pub log_bloom: Bloom,
	pub difficulty: U256,
	pub number: u64,
	pub gas_limit: U256,
	pub gas_used: U256,
	pub timestamp: u64,
	pub extra_data: Vec<u8>,
	pub mix_hash: H256,
	pub nonce: H64,

	pub hash: Option<H256>,
	pub partial_hash: Option<H256>,
}

impl BlockHeader {
	pub fn extra_data(&self) -> H256 {
		let mut data = [0u8; 32];
		data.copy_from_slice(self.extra_data.as_slice());
		H256(data.into())
	}

	fn stream_rlp(&self, stream: &mut RlpStream, partial: bool) {
		stream.begin_list(13 + if !partial { 2 } else { 0 });

		stream.append(&self.parent_hash);
		stream.append(&self.uncles_hash);
		stream.append(&self.author);
		stream.append(&self.state_root);
		stream.append(&self.transactions_root);
		stream.append(&self.receipts_root);
		stream.append(&self.log_bloom);
		stream.append(&self.difficulty);
		stream.append(&self.number);
		stream.append(&self.gas_limit);
		stream.append(&self.gas_used);
		stream.append(&self.timestamp);
		stream.append(&self.extra_data);

		if !partial {
			stream.append(&self.mix_hash);
			stream.append(&self.nonce);
		}
	}
}

impl RlpEncodable for BlockHeader {
	fn rlp_append(&self, stream: &mut RlpStream) {
		self.stream_rlp(stream, false);
	}
}

impl RlpDecodable for BlockHeader {
	fn decode(serialized: &Rlp) -> Result<Self, RlpDecoderError> {
		let mut block_header = BlockHeader {
			parent_hash: serialized.val_at(0)?,
			uncles_hash: serialized.val_at(1)?,
			author: serialized.val_at(2)?,
			state_root: serialized.val_at(3)?,
			transactions_root: serialized.val_at(4)?,
			receipts_root: serialized.val_at(5)?,
			log_bloom: serialized.val_at(6)?,
			difficulty: serialized.val_at(7)?,
			number: serialized.val_at(8)?,
			gas_limit: serialized.val_at(9)?,
			gas_used: serialized.val_at(10)?,
			timestamp: serialized.val_at(11)?,
			extra_data: serialized.val_at(12)?,
			mix_hash: serialized.val_at(13)?,
			nonce: serialized.val_at(14)?,
			hash: Some(keccak256(serialized.as_raw()).into()),
			partial_hash: None,
		};

		block_header.partial_hash = Some(
			keccak256({
				let mut stream = RlpStream::new();
				block_header.stream_rlp(&mut stream, true);
				stream.out().as_slice()
			})
			.into(),
		);

		Ok(block_header)
	}
}

// Log

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
	pub address: H160,
	pub topics: Vec<H256>,
	pub data: Vec<u8>,
}

impl rlp::Decodable for LogEntry {
	fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
		let result = LogEntry {
			address: rlp.val_at(0usize)?,
			topics: rlp.list_at(1usize)?,
			data: rlp.val_at(2usize)?,
		};
		Ok(result)
	}
}

impl rlp::Encodable for LogEntry {
	fn rlp_append(&self, stream: &mut rlp::RlpStream) {
		stream.begin_list(3usize);
		stream.append(&self.address);
		stream.append_list::<H256, _>(&self.topics);
		stream.append(&self.data);
	}
}

// Receipt Header

#[derive(Debug, Clone, PartialEq, Eq, RlpEncodableDerive, RlpDecodableDerive)]
pub struct Receipt {
	pub status: bool,
	pub gas_used: U256,
	pub log_bloom: Bloom,
	pub logs: Vec<LogEntry>,
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
	let mut buffer = [0u8; 32];
	buffer.copy_from_slice(&sha2_256(data));
	buffer
}

pub fn keccak256(data: &[u8]) -> [u8; 32] {
	let mut keccak = Keccak::v256();
	keccak.update(data);
	let mut output = [0u8; 32];
	keccak.finalize(&mut output);
	output
}

pub fn keccak512(data: &[u8]) -> [u8; 64] {
	let mut keccak = Keccak::v512();
	keccak.update(data);
	let mut output = [0u8; 64];
	keccak.finalize(&mut output);
	output
}