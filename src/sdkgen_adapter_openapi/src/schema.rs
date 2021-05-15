use std::convert::TryFrom;

use openapiv3::{OpenAPI as OpenApi, ReferenceOr, Schema};

use crate::NamedOrAnonymous;

#[derive(Debug)]
pub struct NamedSchema {
    pub name: String,
    pub schema: Schema,
}

#[derive(Debug)]
pub struct SchemaReference(String);

impl SchemaReference {
    pub fn resolve(&self, api: &OpenApi) -> Option<NamedSchema> {
        let components = api.components.as_ref()?;
        let schema_or_reference = components.schemas.get(&self.0)?;

        match schema_or_reference {
            ReferenceOr::Item(schema) => Some(NamedSchema {
                name: self.0.clone(),
                schema: schema.clone(),
            }),
            ReferenceOr::Reference { reference } => SchemaReference::try_from(reference.clone())
                .ok()
                .and_then(|schema| schema.resolve(api)),
        }
    }
}

impl TryFrom<String> for SchemaReference {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let schema_path = "#/components/schemas/";

        if !value.starts_with(schema_path) {
            return Err(format!("Not a schema reference: '{}'.", value));
        }

        let path = value.trim_start_matches(schema_path);

        Ok(Self(path.into()))
    }
}

pub(crate) fn resolve_schema(
    api: &OpenApi,
    schema: ReferenceOr<Schema>,
) -> Option<NamedOrAnonymous<Schema>> {
    match schema {
        ReferenceOr::Item(schema) => Some(NamedOrAnonymous::Anonymous(schema)),
        ReferenceOr::Reference { reference } => SchemaReference::try_from(reference)
            .ok()
            .and_then(|schema| schema.resolve(&api))
            .map(|named_schema| NamedOrAnonymous::Named(named_schema.name, named_schema.schema)),
    }
}
