use enum_kinds::EnumKind;

#[derive(Clone)]
pub struct EmptyStruct;

#[derive(Clone, EnumKind)]
#[enum_kind(VerifierTallyDataType)]
pub enum VerifierTallyData {
    TODO(EmptyStruct),
}
