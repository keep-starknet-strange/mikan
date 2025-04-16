use super::proto;
use super::{address::Address, context::TestContext, height::Height, value::ValueId};
use bincode::de::Decoder;
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use bincode::{impl_borrow_decode, Decode, Encode};
use bytes::Bytes;
use malachitebft_core_types::{NilOrVal, Round, SignedExtension, VoteType};
use malachitebft_proto::{Error as ProtoError, Protobuf};

pub use malachitebft_core_types::Extension;

/// A vote for a value in a round
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vote {
    pub typ: VoteType,
    pub height: Height,
    pub round: Round,
    pub value: NilOrVal<ValueId>,
    pub validator_address: Address,
    pub extension: Option<SignedExtension<TestContext>>,
}

impl Vote {
    pub fn new_prevote(
        height: Height,
        round: Round,
        value: NilOrVal<ValueId>,
        validator_address: Address,
    ) -> Self {
        Self {
            typ: VoteType::Prevote,
            height,
            round,
            value,
            validator_address,
            extension: None,
        }
    }

    pub fn new_precommit(
        height: Height,
        round: Round,
        value: NilOrVal<ValueId>,
        address: Address,
    ) -> Self {
        Self {
            typ: VoteType::Precommit,
            height,
            round,
            value,
            validator_address: address,
            extension: None,
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        Protobuf::to_bytes(self).unwrap()
    }
}

impl malachitebft_core_types::Vote<TestContext> for Vote {
    fn height(&self) -> Height {
        self.height
    }

    fn round(&self) -> Round {
        self.round
    }

    fn value(&self) -> &NilOrVal<ValueId> {
        &self.value
    }

    fn take_value(self) -> NilOrVal<ValueId> {
        self.value
    }

    fn vote_type(&self) -> VoteType {
        self.typ
    }

    fn validator_address(&self) -> &Address {
        &self.validator_address
    }

    fn extension(&self) -> Option<&SignedExtension<TestContext>> {
        self.extension.as_ref()
    }

    fn take_extension(&mut self) -> Option<SignedExtension<TestContext>> {
        self.extension.take()
    }

    fn extend(self, extension: SignedExtension<TestContext>) -> Self {
        Self {
            extension: Some(extension),
            ..self
        }
    }
}

impl Protobuf for Vote {
    type Proto = super::proto::Vote;

    fn from_proto(proto: Self::Proto) -> Result<Self, ProtoError> {
        Ok(Self {
            typ: decode_votetype(proto.vote_type()),
            height: Height::from_proto(proto.height)?,
            round: Round::new(proto.round),
            value: match proto.value {
                Some(value) => NilOrVal::Val(ValueId::from_proto(value)?),
                None => NilOrVal::Nil,
            },
            validator_address: Address::from_proto(
                proto
                    .validator_address
                    .ok_or_else(|| ProtoError::missing_field::<Self::Proto>("validator_address"))?,
            )?,
            extension: Default::default(),
        })
    }

    fn to_proto(&self) -> Result<Self::Proto, ProtoError> {
        Ok(Self::Proto {
            vote_type: encode_votetype(self.typ).into(),
            height: self.height.to_proto()?,
            round: self.round.as_u32().expect("round should not be nil"),
            value: match &self.value {
                NilOrVal::Nil => None,
                NilOrVal::Val(v) => Some(v.to_proto()?),
            },
            validator_address: Some(self.validator_address.to_proto()?),
        })
    }
}

fn encode_votetype(vote_type: VoteType) -> proto::VoteType {
    match vote_type {
        VoteType::Prevote => proto::VoteType::Prevote,
        VoteType::Precommit => proto::VoteType::Precommit,
    }
}

fn decode_votetype(vote_type: proto::VoteType) -> VoteType {
    match vote_type {
        proto::VoteType::Prevote => VoteType::Prevote,
        proto::VoteType::Precommit => VoteType::Precommit,
    }
}

impl Encode for Vote {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.height.as_u64().encode(encoder)?;
        self.round.as_u32().encode(encoder)?;

        unsafe { std::mem::transmute::<VoteType, u8>(self.typ) }.encode(encoder)?;

        self.validator_address.into_inner().encode(encoder)?;
        match &self.value {
            NilOrVal::Nil => None,
            NilOrVal::Val(v) => Some(v.as_u64()),
        }
        .encode(encoder)?;
        // Don't encode the extension field at all
        Ok(())
    }
}

impl<Context> Decode<Context> for Vote {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let height = Height::new(u64::decode(decoder)?);
        let round = match Option::<u32>::decode(decoder)? {
            Some(val) => Round::new(val),
            None => Round::Nil,
        };

        let typ = unsafe { std::mem::transmute::<u8, VoteType>(u8::decode(decoder)?) };

        let validator_address = Address::new(<[u8; 20]>::decode(decoder)?);
        let value = match Option::<u64>::decode(decoder)? {
            Some(val) => NilOrVal::Val(ValueId::new(val)),
            None => NilOrVal::Nil,
        };

        Ok(Vote {
            height,
            round,
            typ,
            validator_address,
            value,
            extension: None,
        })
    }
}
impl_borrow_decode!(Vote);

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vote() -> Vote {
        Vote::new_prevote(
            Height::new(100),
            Round::new(2),
            NilOrVal::Val(ValueId::new(3)),
            Address::new([3u8; 20]),
        )
    }

    #[test]
    fn test_vote_bincode_roundtrip() {
        let vote = create_test_vote();
        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(&vote, config).unwrap();
        let (decoded, _): (Vote, _) = bincode::decode_from_slice(&encoded, config).unwrap();
        assert_eq!(vote, decoded);
    }

    #[test]
    fn test_vote_bincode_with_value() {
        let mut vote = create_test_vote();
        vote.value = NilOrVal::Val(ValueId::new(4));

        let encoded = bincode::encode_to_vec(&vote, bincode::config::standard()).unwrap();
        println!("Encoded bytes with value: {:?}", encoded);
        let (decoded, _): (Vote, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(vote, decoded);
        assert_eq!(vote.value, decoded.value);
    }
}
