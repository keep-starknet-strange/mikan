use crate::context::TestContext;
use crate::height::Height;
use crate::value::ValueId;
use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    impl_borrow_decode, Decode, Encode,
};
use malachitebft_core_types::{NilOrVal, Round, SignedExtension, VoteType};
use malachitebft_test::Address;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Vote {
    pub validator: Address,
    pub signature: Vec<u8>,
    pub block: usize,
    pub height: Height,
    pub round: Round,
    pub typ: VoteType,
    pub validator_address: Address,
    pub value: NilOrVal<ValueId>,
    pub extension: Option<SignedExtension<TestContext>>,
}

impl Vote {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        validator: Address,
        sig: Vec<u8>,
        block: usize,
        height: Height,
        round: Round,
        typ: VoteType,
        validator_address: Address,
        value: NilOrVal<ValueId>,
        extension: Option<SignedExtension<TestContext>>,
    ) -> Self {
        Self {
            validator,
            signature: sig,
            block,
            height,
            round,
            typ,
            validator_address,
            value,
            extension,
        }
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

impl Encode for Vote {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.validator.into_inner().encode(encoder)?;
        self.signature.encode(encoder)?;
        self.block.encode(encoder)?;
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
        let validator = Address::new(<[u8; 20]>::decode(decoder)?);

        let signature = Vec::<u8>::decode(decoder)?;
        let block = usize::decode(decoder)?;
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
            validator,
            signature,
            block,
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
    use malachitebft_core_types::VoteType;

    fn create_test_vote() -> Vote {
        Vote::new(
            Address::new([1u8; 20]),
            vec![2u8; 64],
            1,
            Height::new(100),
            Round::new(2),
            VoteType::Prevote,
            Address::new([3u8; 20]),
            NilOrVal::Nil,
            None,
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
