use heck::CamelCase;
use sdkgen_core::CasingRules;

pub struct CsharpCasingRules;

impl CasingRules<String> for CsharpCasingRules {
    fn to_type_name_case(&self, value: String) -> String {
        value.to_camel_case()
    }

    fn to_record_member_case(&self, value: String) -> String {
        value.to_camel_case()
    }

    fn to_function_name_case(&self, value: String) -> String {
        value.to_camel_case()
    }
}
