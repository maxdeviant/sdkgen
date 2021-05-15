use heck::{CamelCase, MixedCase};
use sdkgen_core::CasingRules;

pub struct TypeScriptCasingRules;

impl CasingRules<String> for TypeScriptCasingRules {
    fn to_type_name_case(&self, value: String) -> String {
        value.to_camel_case()
    }

    fn to_record_member_case(&self, value: String) -> String {
        value.to_mixed_case()
    }

    fn to_function_name_case(&self, value: String) -> String {
        value.to_mixed_case()
    }
}
